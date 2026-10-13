# EmuSen_Gpu — one Vulkan compute device for any core that draws on a GPU

*Started 2026-10-13.* `emusen-gpu`, `EmuSen/Cores/Shared/emusen-gpu/`: finding a Vulkan device, or saying plainly why
there is none; storage buffers in host or device memory; compute programs; and submissions that are waited for, left
pending or sent ahead. It is MarsRT's `rdp/gpu/device.rs` (`Mars_Native.md` §6.4.2), itself C#'s `GpuDevice.cs`
(`Mars_Gpu.md` §1, §14.2), moved out of the N64 core so that Diamond's enhanced renderer (`Diamond_Plan.md` §6.2) and
any later core use one device layer rather than a copy.

*Decided 2026-10-13 by the tester:* the device moves into a shared crate that MarsRT and Diamond both use, as the step
towards the one Vulkan device the LunaP kit plans (`EmuSen_LunaP_Kit.md` §5.2: `lunap-gpu` makes the device and lends
it, or imports one), keeping `Diamond_Plan.md` §11 Q14(c)'s decision that the renderer runs on the device MarsRT's GPU
path creates until the kit's exists. The layering decided with it has three levels: (1) this crate, for any core that
draws on a GPU; (2) the rules around an enhanced picture (a skippable state block, an "off is bit-identical" check,
the settings' category), which go where their jobs already live and are Diamond's stage E1; (3) each console's
renderer and shaders, which stay with the console.

The marks are `Diamond_Plan.md`'s: **[M]** measured on 2026-10-13 with the tool named; **[R]** read from a source;
**[A]** argued; **[C]** a choice.

## 1. Its one job, and what it leaves to its users

**In it**: the instance and the choice of a physical device (by type, or by a part of its name); one logical device
with one compute queue; storage buffers in two kinds of memory, the host's (mapped for the buffer's life) and the
device's own (reached by a copy); compute programs whose storage buffers sit at bindings 0 to n−1 of set 0 with one
block of push constants; the three kinds of submission of `Mars_Gpu.md` §14.2 and `Mars_Performance.md` §42 (waited
for, one pending, two ahead); and the commands a submission may hold, a fill, a copy and a dispatch, each followed by
a barrier that orders it before everything later (§3.3).

**Not in it**: any shader, any knowledge of what a buffer holds, any console. MarsRT keeps `rdp/gpu/mod.rs` (the
rasteriser: its memory mirror, rows, 64 by 1 tiles, staging slots, the scan and the average), `rdp/gpu/record.rs` (the
RDP's records), and the five SPIR-V binaries it takes from the C# core's tree [R]. Nothing in the moved file named the
N64: it was searched for every RDP, RDRAM, tile, scan and Mars identifier, and the only one was the device variable's
name (§4) [R].

**Dependencies**: `ash` 0.38 with its default `loaded` feature, so the loader is opened at run time by `libloading`
and a machine without Vulkan gets a report and never a link failure (`Mars_Native.md` §6.4.2). The version is the one
wgpu-hal uses (`EmuSen_LunaP_Kit.md` §5.2), and must stay so while a lent device is planned (§5). Nothing else: not
`emusen-native`, since nothing here is a core's state or ABI.

## 2. What moved, and how MarsRT reaches it

All of `device.rs`, 555 lines, moved as it stood: the enumeration and ranking (discrete, integrated, virtual, other;
the portability extensions for MoltenVK, `Mars_Gpu.md` §4), `GpuDevice`, `GpuBuffer`, `GpuProgram`, `Pending`,
`Submission`, the slots ahead and `Commands`. Three changes were made to its surface on the way, each making it
narrower, and none to what it does:

- **No Vulkan type crosses it.** `GpuBuffer::handle` was public; no caller outside the file read it [R], so it is the
  crate's own. With it, MarsRT no longer names `ash` at all and its manifest drops the dependency; `ash` reaches
  MarsRT's library only through this crate.
- **Fields became methods.** `GpuDevice::{name, software, max_buffer_bytes}` and `GpuBuffer::bytes` were public
  fields a caller could have written; they are now read-only methods. `GpuProgram`'s `buffers` and `push_bytes` had no
  reader outside the file and are private.
- **`finished(Submission)` lost its `#[cfg(test)]`.** It was MarsRT's tests' look at a fence, which waits for nothing;
  a `cfg(test)` item in another crate is not compiled when MarsRT's tests are, so it is now an ordinary method. It is
  a query of a fence's status and changes nothing.

MarsRT's `rdp/gpu/mod.rs` names the crate's items where it named `device::`'s, and re-exports `GpuDevice` and `Pending`
for its own callers (`memory/dp.rs`, its tests), so no other file of MarsRT changed beyond the methods above.

## 3. The surface, version 1.0

Rule **R** of `EmuSen_Surfaces.md` §6.1: an internal Rust API, a path dependency built from the same commit, guarded
by its users' tests. It is nonetheless numbered, so that a second user can rely on its growing by addition: the
crate's version is 1.0.0; an addition (a lent device, images) is a minor; a change or removal of anything below is a
major, and none is planned.

| Item | What it is |
|---|---|
| `GpuDevice::device_names() -> Vec<String>` | every compute-capable device the loader offers, in the order it would be chosen; empty with no loader or no driver, which is how a test stands down |
| `GpuDevice::try_create(Option<&str>) -> Result<GpuDevice, String>` | the first device by rank, or the first whose name contains the given text (any case); with `None`, the variable of §4; the error is a sentence for the player's report |
| `GpuDevice::{name, software, max_buffer_bytes, report}` | the device's name; whether it is a CPU implementation (llvmpipe); the most one storage buffer may bind; `"{name}, Vulkan {major}.{minor}"` |
| `Where::{Host, Device}`, `GpuDevice::create_buffer(bytes, Where)`, `destroy_buffer` | a storage buffer, also a copy's source and target; host memory prefers cached types, since a readback from write-combined memory crawls |
| `GpuBuffer::{bytes, slice::<T>, slice_mut::<T>}` | its size; a host buffer's mapped bytes as values (a device buffer panics: it is reached by a copy) |
| `GpuDevice::create_program(spirv, buffers, push_bytes)`, `create_program_with_sets(…, sets)`, `destroy_program` | a compute program, entry `main`, with `sets` descriptor sets (at most `AHEAD`), one for each submission that may be out with it at once |
| `GpuDevice::submit(record)` | records, submits and waits |
| `GpuDevice::submit_pending(record)`, `pending() -> Arc<Pending>`, `wait_for_pending`, `Pending::wait` | one submission left out, waited for by the next or by a thread that never submits (the presenter's) |
| `AHEAD`, `GpuDevice::submit_ahead(slot, record)`, `wait_ahead(slot)` | two slots submitted without waiting, each waited for only when it is used again |
| `GpuDevice::finished(Submission) -> bool`, `Submission::{Now, Pending, Ahead(slot)}` | whether a submission has finished, waiting for nothing |
| `Commands::{fill, copy, dispatch, dispatch_in, submission}` | what a recording holds: a fill of a whole buffer with a word, a copy between buffers at offsets, a dispatch of a program with its buffers and push constants (in descriptor set 0 or a later one), and which submission it is |
| `DEVICE_VARIABLE` | `"EMUSEN_GPU_DEVICE"` (§4) |

**The rules a user keeps**, unchanged from MarsRT and stated once here [R `device.rs`]:

1. A descriptor set is written when a dispatch is recorded. A program's set may be reused only once the last
   submission that used it has finished; that is what `create_program_with_sets` and the slots are for.
2. Every command is followed by a memory barrier from compute and transfer writes to compute and transfer reads and
   writes, and a barrier's second scope is everything later in submission order, other submissions included
   (`Mars_Gpu.md` §14.2). Every submission ends with a barrier to the host, because a fence makes the device's
   accesses complete but not visible to a host read of mapped memory.
3. A host buffer read or written by a submission is the submission's until it has finished.
4. One thread submits. `Pending::wait` is the only call another thread may make, and only while nothing submits.

## 4. Choosing a device, and the variable's name

The variable was `EMUSEN_MARS_GPU_DEVICE`. It is now **`EMUSEN_GPU_DEVICE`**, in this crate and in C#'s `GpuDevice.cs`,
so that the one knob read by both N64 cores (`EmuSen_Surfaces.md` §4.10) stays one knob under a name that is not one
console's.

**Where the old name was set or read** [R, searched 2026-10-13 over the repository and `~/.cache/emusen/probe`, every
text file outside build output]: read in two places, `Mars - N64/Rdp/Gpu/GpuDevice.cs` and MarsRT's `device.rs`;
named in `Mars_Gpu.md` §1 and §3 and in `EmuSen_Surfaces.md` §4.10; set by **nothing**: no test, no script, no
frontend, no CI workflow. The probe folders hold it only inside copies of those same sources and pages. The test
selector `EMUSEN_MARS_GPU_TEST_DEVICES` is a different variable, the N64 suites' own choice of which devices their
tests run on, and stays theirs.

**The old name is not kept** [C]. Nothing in the project or its measurement scripts sets it, so an alias would serve
only a player who had exported it by hand, a choice of adapter on a machine with two; such a player's machine then
falls back to the ranked choice (the discrete card), which is the default anyway and is reported in the device's
report. One name read in one place is the simpler arrangement.

`EMUSEN_SLANG_GPU_DEVICE` (`Platform/serenity/src/vulkan.rs`, the slang chain's device) is not this variable and is
not changed: it chooses a different device made by a different component (§6.3), and the two meet only in the kit's
one device (§5).

## 5. A lent device: the seam, stated and not built

The kit's `lunap-gpu` will either make the device and lend it, or import one (`EmuSen_LunaP_Kit.md` §5.2). This crate
must then draw on a device it did not create, without its users' code changing. The seam is the constructor and
nothing else [A]:

- **Entry.** One more constructor beside `try_create`, of the form `GpuDevice::lent(Lent) -> Result<GpuDevice,
  String>`, where `Lent` carries the lender's `ash::Entry`, `ash::Instance`, physical device, `ash::Device`, a queue
  family with a compute capability and its queue, and the lender's lock on that queue. It fills the same fields
  `open` fills today, reads the memory properties and limits from the physical device as `open` does, and makes its
  own command pool, command buffers and fences on the lent device. Every other item of §3 is unchanged, so a user
  that holds a `GpuDevice` cannot tell how it was made. This is an addition, version 1.1.
- **What the crate then owns, and what it does not.** On a lent device `Drop` destroys only what the crate made (its
  fences, its command pool) and waits on its own fences rather than calling `vkDeviceWaitIdle`, which would wait for
  the lender's work too; the device and the instance are the lender's. Today `Drop` ends by destroying both, the one
  place an ownership flag must be read.
- **The queue is shared.** Vulkan requires a queue's submissions to be synchronised by the application, which the
  kit's plan already states. `GpuDevice::record` makes the crate's one `vkQueueSubmit` call; a lent queue's lock is
  taken around that call and nowhere else. A crate-made device needs no lock, since one thread submits (rule 4).
- **What crosses then.** `ash` types appear in the surface for the first time, which is why the version must remain
  the one wgpu-hal uses; a lender on another version is a compile error, not a run-time fault.
- **What a lender must provide**: Vulkan 1.1 or later, a queue family with compute, and the portability subset
  enabled when the implementation offers it (as `open` enables it for MoltenVK). The constructor checks the first two
  and refuses with a reason.

Nothing of this is built: there is no lender, and the kit's half is the kit's.

## 6. A second user: what Diamond's enhanced renderer will need

Read from `Diamond_Plan.md` §6.2, §6.3 and its stages E3 to E5 [R], against the surface as built.

### 6.1 What the plan asks of the device

- **E3**, the renderer at one times, consuming the GP0 stream beside the software rasteriser, its picture equal to
  the software rasteriser's display "exactly or within a stated count of pixels". Its gate compares pictures, so the
  renderer's picture must reach the host: a copy into a host buffer, which exists.
- **E4**, sub-pixel vertices, perspective-correct texturing and seam fixes: more precise inputs to the same renderer.
  Nothing new of the device.
- **E5**, internal resolution at 2× and 4×, true colour, texture filtering: memory at N² (a buffer at 4× of the
  1024 by 512 VRAM in 32-bit words is 32 MiB, well inside the 256 MiB MarsRT's mirror takes at four, and checked
  against `max_buffer_bytes` as MarsRT checks its own); 24-bit output; filtering "with clamping to the primitive's
  texture rectangle", which is arithmetic in a shader whichever form the renderer takes.
- §6.2's rules: the renderer owns only the presented image and never machine state; lavapipe runs the same code on a
  machine without a GPU (`software()` says when it is so, for the report's honesty about cost).

### 6.2 The form the renderer takes decides what is missing

The plan does not say whether the enhanced renderer is a compute rasteriser over a mirror of VRAM, as MarsRT's is, or
a graphics pipeline drawing into images. That is E3's design decision, and it changes this crate's answer:

- **As compute over a mirror** [A, the likelier]: the PlayStation draws into one 1024 by 512 surface from which it
  also reads its textures and palettes, copies within it, and tests a mask bit stored in it. That is `Mars_Gpu.md`
  §5.1's argument for a mirror over images, nearly word for word, and it holds more strongly here, since the
  PlayStation has one surface where the N64 has arithmetic on addresses. Built that way, **E3 to E5 need nothing this
  crate lacks**: storage buffers, compute programs, copies, the readback, the three kinds of submission.
- **As a graphics pipeline**: images (an image at N× as a colour target, perhaps a depth image for E6), image views and
  layout transitions, samplers if the filtering is fixed-function, graphics pipelines with vertex input and blending
  (the PlayStation's four semi-transparency modes are blend equations, B−F among them as a reverse subtract), dynamic
  rendering or render passes, and a graphics-capable queue family. Each is an addition: new types and new
  `Commands` methods beside the present ones, and a program constructor taking a list of bindings beside
  `create_program`'s storage-buffer-only one. **One internal change would follow**: rule 2's barrier names the compute
  and transfer stages only, and graphics work needs its stages in it. That widens a barrier MarsRT also records, so it
  would be measured on MarsRT's bench when made, not assumed free.

### 6.3 What neither form needs yet

- **The presented image handed to the host without a readback.** For Mars it was measured and not built
  (`Mars_Gpu.md` §16); a core's picture today leaves through the core ABI as pixels. It is the kit's lent device that
  makes it possible (§5), and the decision is to be measured again then.
- **A second device in one process.** Each core opens its own device when its renderer is on; the frontends run one
  core at a time.
- **Serenity's device.** `Platform/serenity/src/vulkan.rs` already has images and a graphics queue, for the slang
  chain. It belongs to the platform library, behind interface B, and a core must build and run with no frontend, so a
  core cannot take it. If images are added here, that code is the project's own and may be read for how; it is not
  emulator source.

### 6.4 The next step

Nothing is added now, because nothing is plainly missing for E3 in the likelier form. Two things are E3's first work,
not this crate's: **deciding the renderer's form** in writing (§6.2), and **a route from shader source to SPIR-V for
a core with no C# tree**. MarsRT takes its binaries from the C# core's `Rdp/Gpu/Shaders/`, compiled and checked by
`MarsGpuDeviceTests` through shaderc in WiseMan (`Mars_Gpu.md` §2); Diamond has no C# core, so its `.comp` sources need
the same record-and-check test of their own. If E3 decides on the graphics form, the additions of §6.2 come first,
as a minor of this surface, with MarsRT's bench run across the barrier's change.
