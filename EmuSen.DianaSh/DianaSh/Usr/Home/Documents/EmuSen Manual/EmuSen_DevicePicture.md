# EmuSen_DevicePicture — the game's picture on one Vulkan device, Skia drawing only the interface

A design (2026-10-15), written before any of its code, and the record of each step as it is built. It answers one
question: how the game's picture, the screen filters over it and its hand-over to the compositor move off Skia in
the C# frontends of today, onto one Vulkan device, so that Skia is left drawing the interface.

**Decided 2026-10-15 by the tester.** The renderer and the presentation move off Skia. The whole answer is the LunaP
kit's (`EmuSen_LunaP_Kit.md` §5: one device in `lunap-gpu`, Vello for the interface, Skia gone at K9), which is months
away and keeps its own track. Accepted now, as a staged project with this design first, is the shortcut inside
Avalonia, which is the kit's GPU half done early: its prerequisite P4 (§8 there) and §5.2's one device. Three parts:

1. **The built-in filters as Vulkan passes**, run by the RetroArch runtime's chain (`EmuSen_Serenity.md` §7), each
   matched to its SkSL output pixel for pixel or within a stated, argued tolerance on rendered real frames before it
   replaces the SkSL one.
2. **No readback**: the chain's finished image handed to Avalonia's compositor with the GPU synchronised by
   semaphores, and today's readback chosen at run time where that cannot be done.
3. **A GPU core hands over its image**: a core ABI minor in which the host lends its device, the core renders into
   memory it hands back with a semaphore, and a core without it keeps handing pixels; then Diamond's enhanced renderer
   on it, its picture no longer a frame late.

**Reading order.** §1 is where things stand, read and probed on 2026-10-15. §2 to §6 are the design: the device's
ownership, the filters, the compositor, the ABI minor, and what moves into `emusen-gpu`. §7 is Diamond and MarsRT,
§8 the steps with the measurement each must show, §9 the risks and §10 the questions. §11 on are the steps as built.

**Marks.** **[M]** measured or probed on 2026-10-15 on the development desktop (Ryzen 7 7700X, RX 6800 under RADV and
radeonsi, Fedora 44, KDE under Wayland with Mistress's windows on XWayland), with how; **[R]** read from a source
named; **[A]** argued; **[C]** a choice.

---

## 1. Where things stand (2026-10-15)

### 1.1 Three devices and three paths

A frontend process may hold three GPU contexts today [R]:

| Context | Made by | API | Draws |
|---|---|---|---|
| Avalonia's compositor | Avalonia, under `UseX11()` with GLX by default (`LunaApp.cs`) | OpenGL, radeonsi | the interface, and through Skia the game's picture and the SkSL filters (`GameFrameControl`, `EmuSen_Serenity.md` §2, §3) |
| Serenity's slang device | the platform library, `Platform/serenity/src/vulkan.rs`, one per process (`SlangRunner.Shared`) | Vulkan 1.1, one graphics queue | a RetroArch preset, its picture read back into host memory and uploaded again by Skia (§7.4, §9.1 there) |
| A core's device | `emusen-gpu`, opened by MarsRT's GPU path or Diamond's enhanced renderer (`EmuSen_Gpu.md`) | Vulkan 1.1, one compute queue | the core's own picture, read back into host slots and handed over as pixels through `frame_copy` |

So the game's picture crosses the bus up to three times: a GPU core reads its picture back, Skia uploads it, and a
RetroArch preset reads it back again and Skia uploads that. §8.3 of `EmuSen_Serenity.md` measured what the second
half costs a light preset at 1080p (1.8 ms of the render thread after §9's levers, most of it the readback, two
waits and the upload); `Diamond_Enhancements.md` §38.1 pays a frame of latency so that its readback never stalls the
emulation thread.

### 1.2 What Avalonia 12.1 offers, probed in a window

A probe (`~/.cache/emusen/probe/gpu/interop/`, a 320 × 200 window of its own, never Mistress) asked the compositor
for `ICompositionGpuInterop` under each X11 rendering mode [M]:

| Mode | Image handles | Semaphore handles | Synchronisation | `DeviceUuid` |
|---|---|---|---|---|
| GLX (the frontends' default) | `VulkanOpaquePosixFileDescriptor` | `VulkanOpaquePosixFileDescriptor` | `Semaphores` | `00000000030000000000000000000000` |
| EGL | none | none | — | none |
| Vulkan | `VulkanOpaquePosixFileDescriptor` | `VulkanOpaquePosixFileDescriptor` | `Semaphores` | `00000000030000000000000000000000` |

RADV reports the RX 6800's `deviceUUID` as `00000000-0300-0000-0000-000000000000` and the integrated Raphael GPU's as
`…-0f00-…` (`vulkaninfo`), so the GL compositor's UUID names the card a Vulkan device must be made on for an import
to be valid. EGL offers nothing, which is the first reason the hand-over must be chosen at run time.

What the API does with an image, read from Avalonia 12.1.0's own assemblies (`ServerCompositionDrawingSurface`,
`Avalonia.Skia.GlSkiaImportedImage`) [R]: `UpdateWithSemaphoresAsync` is applied on the compositor's thread, where it
waits on the first semaphore, **copies the imported image into a texture it creates for the purpose**
(`CopyToNewTexture`), signals the second semaphore and flushes; the surface then draws that copy. So the imported image
is free for reuse once the second semaphore is signalled, and the surface keeps showing its copy for as long as no
newer update arrives. The price is one GPU copy and one texture made per update, inside Avalonia.

### 1.3 The queues the device offers

The RX 6800 under RADV has one queue in its graphics family and four in a compute-and-transfer family; the integrated
GPU the same; llvmpipe one in all [M, `vulkaninfo`]. All three support timeline semaphores. A device can therefore lend
a core a queue of its own on both of this machine's GPUs, and on llvmpipe it cannot.

---

## 2. The device: whose it is, and why one

**The device is Serenity's** [C]: the one the platform library makes for the process (`emusen-serenity`'s `Device`,
`SlangRunner.Shared`). Everything the frontend draws on the GPU is drawn on it, the RetroArch presets, the built-in
filters (§3) and the plain picture when it is presented by the device (§4), and it is the device lent to a core (§5).

**Why one device and not one per part** [A]. A core's picture and the filter that draws it are then on one `VkDevice`:
the chain reads the buffer the core wrote, with no export or import between them, and the semaphores between the
core's queue and the presenter's are ordinary semaphores of one device. Two devices on one physical GPU could share by
opaque file descriptors as well, but every object crossing would be exported and imported, the two would have to be
on the same physical device (checked by UUID), and the copies across would be the only thing gained by keeping them
apart. This is §5.2 of the kit's plan, in the order the kit gives it: one device, the game's picture never leaving it.

**Why the frontend's device and not the core's** [A]. The presenter exists before any game and outlives each; a
frontend runs one core at a time; and the kit's `lunap-gpu` makes the device and lends it, the same direction. A core
must build and run with no frontend (`EmuSen_Gpu.md` §6.3), so it never reaches for the frontend's device: it is
*lent* one through the core ABI (§5), and makes its own, as today, when nothing is lent (a test, a tool, the kit, an
older host, a host with no device).

**What the device gains, each optional and chosen at run time** [C]:

| Gains | For | Without it |
|---|---|---|
| `VK_KHR_external_memory_fd`, `VK_KHR_external_semaphore_fd` | exporting the presented image and its semaphores to the compositor (§4) | the readback (§4.4) |
| `timelineSemaphore` (Vulkan 1.2's feature, or `VK_KHR_timeline_semaphore` on 1.1) | a core's picture handed with a value instead of a pairing (§5.3) | nothing is lent; the core hands pixels |
| a second queue in a family with compute | lent to a core, so that no lock crosses the ABI (§5.2) | nothing is lent |

**Avalonia's context is not a Vulkan device under GLX** [R, §1.2], so "one device" ends at the compositor: the image
crosses into GL by an opaque descriptor, both sides on the same physical GPU by UUID. Avalonia's X11 Vulkan mode
offers the same interop (§1.2), and its `VulkanOptions.CustomSharedDevice` would let Avalonia draw on Serenity's
device itself, which is one device to the swapchain; that changes the renderer under every window of both frontends
and is Q1 (§10).

**Where one device is not possible** the parts fall back separately: no second queue, or no timeline semaphores, and
a core keeps its own device and hands pixels; no external extensions, or a compositor that offers no import, and the
chain's picture is read back. Each is reported once, in the log the slang device already writes to.

---

## 3. The built-in filters as passes of the chain

### 3.1 The method

Every built-in filter (Scanlines and Simple CRT, `BuiltInShaders`; CRT (Lottes), `CrtFilters`; the six handheld LCDs,
`HandheldFilters`; the modelled CRT's three tiers, `CrtFilter`, `EmuSen_CRT.md`) is turned into a **slang preset**: a
`.slangp` and a `.slang` per pass, written by Serenity into a folder of its own under `home/Shaders/` and run by
`SlangChain` exactly as a downloaded preset is [C]. Files are the interface of §7 of `EmuSen_Serenity.md`: the chain
needs no second way in, both implementations of it (the platform library's and the C# one behind the switch, 
`EmuSen_RustPlatform.md` §17) run the built-ins with no change of their own beyond §3.4's, and a player's own preset
can `#reference` a built-in's file, which is how the built-in CRT stacks with any RetroArch pass.

**Done in two stages, so that the SkSL stays the reference until the passes are proven** [C]:

1. **A translation at build time.** `BuiltInPreset` (Serenity) writes each `FilterPass`'s SkSL into a Vulkan GLSL 4.50
   pass: a fixed preamble that names SkSL's types as GLSL's (`float2` as `vec2`, `half4` as `vec4`, and the rest),
   each `uniform shader x;` as a sampler with a function `x_eval`, each `x.eval(` as `x_eval(`, each uniform as a
   member of the pass's block under the slang semantic that means the same (§3.2), the SkSL `main(float2 coord)` as a
   function the pass's `main` calls with `gl_FragCoord.xy`, and a vertex stage of its own. The SkSL text is then the
   one source of both paths, and a test draws both from it.
2. **The text converted, once proven.** The SkSL in `CrtFilter.cs`, `CrtFilters.cs`, `HandheldFilters.cs` and
   `BuiltInShaders.cs` is rewritten in GLSL, the translation's preamble and rewrites dropped, and the Skia path
   (`FilterChain`, the runtime effects of `GameFrameControl`) removed. The conversion is proven the cheap and exact
   way: every pass of every variant, preprocessed by glslang from the old text with the preamble and from the new text
   without it, gives the same tokens. Removing the Skia path is Q2 (§10).

### 3.2 What each name means on the chain

| `FilterChain` (`EmuSen_Serenity.md` §3.2, §3.9) | On the chain |
|---|---|
| `coord`, the destination pixel | `gl_FragCoord.xy`, the pass's own pixel centre, as Skia's local coordinate is for a pass drawn into its own surface |
| `source` | `Source` |
| `original` | `Original`, its rows repeated on the device for a filter that is not `RowsOnce` (§9.2 there), as `FilterChain`'s stretch matrix shows them |
| `history1` … `historyN` | `OriginalHistory1` … `N` |
| `feedback` | the pass's alias followed by `Feedback` |
| a named pass, and `nameSize` | the alias, and `aliasSize.xy` |
| `raw`, `rawSize` | a lookup image named `raw`, replaced in place with each frame, and its size (§3.4) |
| `inputSize`, `originalSize`, `outputSize`, `viewportSize` | `SourceSize.xy`, `OriginalSize.xy`, `OutputSize.xy`, `FinalViewportSize.xy` |
| `frameCount` | `float(FrameCount)`, the preset's `frame_count_mod` 65536, both counting the frames advanced (§3.5) |
| a filter's parameter, a console's constant | a `#pragma parameter`, its value the preset's own for the console and structural values the preset is written for |
| `pixelScale`, `pixelOrigin`, `crop` | parameters of reserved ids that no window lists, set by the runner at each draw |
| `PassScale.Source`, `Viewport`, `PassSize` | `scale_type` `source`, `viewport` or `absolute`, with `scale_x`, `scale_y` |
| `Float` | `float_framebuffer`: `R16G16B16A16_SFLOAT`, Skia's `RgbaF16` |
| nearest sampling | `texelFetch` at the texel `floor(p)`, clamped to the image, whatever sampler is bound |
| `LinearSource`, `Linear` | `texture()` at `p / size`, the bound sampler linear by libretro's rule for that image (§7.4 there: an output is sampled with the filter of the pass after it), set so for every image some pass reads linearly |
| clamp to edge | `wrap_mode = clamp_to_edge` on every pass (libretro's default is the border) |
| an 8-bit pass: cleared black, drawn with `SrcOver` | the colour written as `vec4(rgb, 1.0)`, which is what `SrcOver` over opaque black stores |
| a float pass, drawn with `Src` | the colour written as it is |
| `RowsToALine`'s reducing pass (`EmuSen_CRT.md` §16) | a first pass of its own at a source scale of 1 / *n*, the preset written again when *n* changes |

Sampling is done by fetch rather than by the bound sampler wherever the SkSL samples nearest, because libretro binds
one sampler per image for every reader and `FilterChain` lets each pass choose per child: a fetch makes the nearest
reads independent of what the rule binds, and the linear reads are then the only ones the rule must serve.

### 3.3 Matched, and the tolerance

**The comparison** [C]. Each filter family in turn (the handheld LCDs, then Lottes and the two simple effects, then
the CRT's three tiers on every console and signal) is drawn by both paths on the RX 6800 from the same frames: Skia
on its GL context through `ShaderBench.Picture` (§8.1 of `EmuSen_Serenity.md`), the chain on the slang device. The
frames are real games' pictures (the frontends' resume screenshots of NES, SNES, Genesis, N64, Game Boy and Game Boy
Color games, copied from the ROM library into scratch and never committed, and Diamond's pictures from its bench
discs) and the suite's synthetic patterns where a test must run without the library. Each picture is compared channel
by channel; what is reported is the largest difference, the count of channels that differ, and where they are.

**The tolerance, argued before it is measured** [A]. Both paths run on the same GPU and the same shader back end
(Mesa's NIR to ACO under radeonsi and RADV); they differ in the front end (Skia's SkSL-to-GLSL generator and the GL
driver's GLSL compiler, against glslang to SPIR-V), so in the order of operations and in which products are fused
into multiply-adds. Every quantity a filter computes is a float32 expression of the same inputs, and a reordering
moves a result by a few units in its last place, which moves an 8-bit channel only where the value lies within that
distance of a rounding boundary: **at most one level**, on a small share of channels. Larger differences can arise
only where an expression feeds a discontinuity: a `floor`, a comparison or a fetch's texel choice whose argument
falls within a last-place unit of a whole number. The stated tolerance is therefore: **every channel within 1 level;
any channel beyond it explained as a discontinuity, by its position and the expression that put it there, and fewer
than one channel in 10⁵**. A filter that fails it is not switched. The measurement in §12 retires or revises this.

**What it does not compare** [A]. A destination rectangle at a fractional position: Skia evaluates the last pass at
the window's device pixels offset by the fraction, while the chain renders whole pixels drawn at the fraction, so the
comparison uses windows whose letterbox falls on whole pixels (1440 × 1080 in 1920 × 1080). And a scaled desktop:
Skia's intermediate viewport passes are in canvas units and the chain's in device pixels, which is the chain's
behaviour for every preset (§7.5 there) and arguably the right one; it is recorded, not matched.

### 3.4 What the chain must learn

One thing, in both implementations: **a lookup image replaced in place**, so that the NES's raw frame (`EmuSen_CRT.md`
§22) reaches a pass each frame with no image made per frame. `ChainLut` on an image of the same size writes into it;
of another size it makes a new one as today. Nothing else: pass sizes, float targets, feedback, history, aliases and
parameters are §7.4's.

### 3.5 Where the semantics differ, and the choice

- **History before there is any.** `FilterChain` reads a missing history frame as the frame itself; the chain's ring
  starts cleared, as RetroArch's does. A handheld LCD therefore blends towards black for its first three frames on
  the chain [A]. Kept [C]: libretro's semantics win for a runtime that runs libretro's presets, and three frames are
  50 ms after a filter is chosen. The comparison starts after the deepest history is full.
- **`frameCount`.** Both count frames advanced since the chain was made; the CRT's phase alternation and the
  PAL switch read its parity, so they agree once both start together, which the comparison arranges.

---

## 4. The hand-over to the compositor

### 4.1 Which path, argued

Two paths exist [R, §8.5.1 of `EmuSen_Serenity.md`, §1.2]:

- **(a) Avalonia's**: the chain's last image exported, imported once with `ICompositionGpuInterop.ImportImage`, and
  shown by a `CompositionDrawingSurface` in a `CompositionSurfaceVisual` under `GameFrameControl`, updated with
  `UpdateWithSemaphoresAsync`.
- **(b) The GL-context import of §8.5.1**: inside `GameFrameControl`'s draw operation, the exported memory imported
  into Avalonia's own GL context with `glImportMemoryFdEXT`, waited for with `glWaitSemaphoreEXT`, and drawn by Skia
  as a texture.

**Chosen: (a)** [C], for four reasons:

1. **It is Avalonia's defined surface.** (b) issues GL calls inside a context Avalonia and Skia own, behind Skia's
   state tracking, with function pointers the frontend loads itself; that is reaching past a surface, which the
   project's rules forbid, and it binds the frontends to GL under Avalonia.
2. **The run-time choice is Avalonia's own answer.** `SupportedImageHandleTypes`, `SupportedSemaphoreTypes`,
   `GetSynchronizationCapabilities` and `DeviceUuid` say, per window, whether (a) can be done on this device; (b) would
   have to probe the GL context for its extensions and guess its device.
3. **It survives a change of Avalonia's renderer.** It works under the Vulkan mode as under GLX (§1.2), which (b)
   cannot, and it is the shape the kit's compositing takes (an imported image with semaphores).
4. **The picture leaves the render thread.** The chain runs where the frame arrives (§4.3), and the compositor's work
   for the picture becomes drawing a texture.

**What (a) costs that (b) does not** [R, §1.2]: Avalonia copies the imported image into a texture it makes for each
update. On the GPU that is a copy of the shown picture, of the order of §8.5.1's device-side copy (0.02 to 0.05 ms at
1080p); on the compositor's thread it is a texture's creation. §13 measures it; if (a) costs the render and UI threads
together more than (b)'s bench figure by more than the copy, the choice is put to the tester again with the numbers.

### 4.2 The synchronisation

**Images.** A ring of three exportable images (`VK_EXTERNAL_MEMORY_HANDLE_TYPE_OPAQUE_FD_BIT`, dedicated memory,
`R8G8B8A8_UNORM`, optimal tiling), each imported once; made again only when the shown rectangle changes size.

**Semaphores, two per image** [C]: *ready*ₖ, signalled by the chain's submission that drew image *k* and waited on by
the compositor before its copy; and *free*ₖ, signalled by the compositor after its copy and waited on by the next
submission that draws image *k*. Both are binary and exported once. Binary semaphores must be signalled before they
are waited on and waited on once per signal, and the compositor signals *free*ₖ only when it applies the update, which
the task `UpdateWithSemaphoresAsync` returns reports. So an image is drawn again only when its last update's task has
completed; when none of the three is free, the frame is not handed over and the next one is. **The CPU never waits for
the GPU on this path**: the chain's submission waits for *free*ₖ on the device, the compositor waits for *ready*ₖ on
the device, and the only test the CPU makes is whether a task has completed.

**The chain with frames in flight** [C]. Today each submission of the chain is waited for (`device.run`). For the
hand-over it is not: what a submission reads from host memory or rewrites per frame (its command buffer, each pass's
uniform buffer and descriptor set, the staging buffer of the picture) is kept twice, and a frame waits only for the
fence of the frame before last before it rewrites them, which on any machine that keeps up has long signalled. Passes
written by one frame and read by the next acquire a subpass dependency into the render pass (from the shader reads to
the colour attachment's write), which §10.2 of `EmuSen_Serenity.md` did not need while each frame was waited for. The
validation layer, with synchronisation validation and its shader-access heuristic, is the gate (§8).

### 4.3 Where it runs

`GameFrameControl` is given the picture on the UI thread (Mistress's `FrameHandOff`, Hotaru's window, `FramePresenter`).
With the device path on, the frame is advanced, the chain run and the update posted there, at most one outstanding
and the newest frame winning, as `Latest<T>` does (§4 of `EmuSen_Serenity.md`); a change of size or of a parameter
runs it again with the same frame, as `SlangRunner` does now. The surface visual is placed at the destination
rectangle of §2.9 and §2.10 there (its offset and size, a clip for a crop), so the shape, the crop and the letterbox
are unchanged; the control's own draw operation draws nothing of the picture while the visual shows it.

### 4.4 The fallback, chosen at run time

The device path is used only when all hold, each checked once per window and logged once when it does not [C]:

1. the compositor offers interop, with `VulkanOpaquePosixFileDescriptor` among both its image and semaphore types and
   `Semaphores` among the image type's synchronisation capabilities;
2. its `DeviceUuid` is the slang device's `deviceUUID`;
3. the slang device has the two external extensions and is the platform library's (the C# device, behind the switch
   at 0, keeps the readback; `EmuSen_RustPlatform.md` §19);
4. every import succeeds.

Otherwise the picture is drawn as today: Skia draws the plain picture, a preset is read back and drawn by Skia (§9.1
of `EmuSen_Serenity.md`), and a built-in filter is drawn by Skia from its SkSL while that text stands (§3.1's first
stage), which costs the render thread 0.09 ms where its readback would cost 1.8 (§8.3 there), and read back once the
text is converted. That is the path on Windows (ANGLE offers D3D11 handles, not Vulkan's),
on macOS (no Vulkan device is shipped there; Q2), under Avalonia's EGL mode, on a machine whose compositor and Vulkan
device are different cards, and with no Vulkan device at all.

### 4.5 The plain picture

With the device path on, **every picture goes through the device**, the plain one included, drawn by a one-pass
built-in that samples nearest or linearly as `BilinearFiltering` says [C]. One path for every picture is simpler than
two, and a core's picture on the device (§5) has no other way to the screen without a readback. Where the device
path is off, Skia draws the plain picture as it does today.

---

## 5. The core ABI minor: the picture on the device

### 5.1 The number, and the form

**1.9**, the next free minor after 1.8 (§17 of `EmuSen_CoreAPI.md`) [R]. Its smallest form [C]:

- a capability bit, **`EMUSEN_CAP_DEVICE_PICTURE`** (bit 21);
- **`emusen_create_params` grows by one field**, `const emusen_device_lend *device` (104 bytes; 96 and 88 still
  accepted), null when nothing is lent;
- **`emusen_device_lend`**, a struct beginning with `size`: the lender's `vkGetInstanceProcAddr`, its `VkInstance`,
  `VkPhysicalDevice` and `VkDevice`, the API version the device was made with, **the queue lent** (family and index,
  used by the core alone), the host's own queue family (for buffers shared between the two), and the host's
  **timeline semaphore `done`**;
- one optional export, **`emusen_core_frame_device(m, emusen_device_frame *out)`**, called after `advance` (and
  `present`), which answers 1 with the frame's picture on the device, 0 when this frame's picture is not there (then
  `frame_copy` has it), or a status. `emusen_device_frame` begins with `size` and holds the `VkBuffer`, its offset,
  the picture's width, height, row stride and format, and the core's **timeline semaphore `ready`** with the value it
  will reach when the picture is written.

### 5.2 Why these and not others

- **A buffer, not an image** [A]. Both GPU cores keep their picture in a storage buffer (MarsRT's scan-out, Diamond's
  `picture`, §16 of `Diamond_Enhancements.md`), and `emusen-gpu` makes buffers only (`EmuSen_Gpu.md` §1). An image
  would need layouts, layout transitions across the ABI and queue-family ownership of an image; a buffer needs none
  of them. The chain copies the buffer into its history image, which it does with a staging buffer today.
- **A queue of the core's own, and no lock** [A]. `EmuSen_Gpu.md` §5 planned the lender's lock on a shared queue; a
  lock crossing the ABI is a callback, which the interface has none of (§6.20 there). A lent queue that only the core
  submits to keeps rule 4 of `emusen-gpu` (one thread submits) per queue. The cost is that a device with one queue
  lends nothing (§1.3).
- **Timeline semaphores, not binary ones** [A]. A binary semaphore must be waited on once for each signal, so a host
  that skipped one picture would break the pairing for good. A timeline value says "every picture up to this one", so
  a host may let pictures go in any number, by one signal of the newest.

### 5.3 The rules

- **The core**: with a device lent and the bit claimed, it may draw on the lent device with the lent queue; the
  picture's memory is a buffer it made with concurrent sharing over its queue's family and the host's; it signals
  `ready` to the frame's value when the picture is written; and it does not write that picture's memory again until
  the host's `done` has reached that value. It never destroys the lender's objects.
- **The host**: it signals `done`, on the device and in order, to the value of every picture it was handed, whether
  it showed the picture or not; it may cover several with one signal of the newest. **A host that stops signalling
  stalls the core's device and then its machine, and corrupts nothing** [A]: that is the failure the rule turns a
  host's mistake into.
- *As built (§15): "handed" is "answered by `frame_device`".* A picture the host never asked for binds the core to
  nothing, so a host that lends and reads `frame_copy` alone is never stalled; and the core's machine stalls only
  once a picture is held past the next two `advance` calls.
- **`frame_copy` still answers** on a frame whose picture is on the device, by waiting for it and reading it back. A
  screenshot, a state's picture, the kit and every tool keep working; they pay a wait where the presenter does not.
- **States** hold nothing of the device; a state saved with a device lent loads without one and the reverse.

### 5.4 Skew, both ways

- **An older host, a newer core.** The host passes 96 or 88 bytes and never calls `frame_device`; the core reads no
  lend and makes its own device or none, and hands pixels, as at 1.8.
- **A newer host, an older core.** The core claims no bit; the host never calls `frame_device`, never lends (a lend in
  a struct the core reads only 96 bytes of is not seen) and takes `frame_copy`, as at 1.8.
- **A newer host with no device to lend** passes null; a core claiming the bit answers `frame_device` with 0 and hands
  pixels.

### 5.5 The kit

**C20**, where the bit is claimed: a machine made with a device the kit lends (two queues on one device, made by the
kit with `ash`) and one made with none run the same image; each frame, the picture on the device (copied by the kit
on its own queue after waiting for `ready`) equals the lendless machine's `frame_copy`, and `frame_copy` on the lent
machine equals both; the kit holds `done` back for a frame and checks that the core's next picture is still right
when it is released; a lend whose struct is too short is refused or ignored, never read past. Where the bit is not
claimed, C2 already checks that the export is absent. A skew fixture (§12.5 there): a host of 1.8 against a core of
1.9, and the reverse, by the struct sizes above. *As built (§15): the lent machine's `frame_copy` must equal the
lendless one's at the same frame or a frame sooner, since a core lent a device may drop §7's frame of latency; each
picture on the device equals the lent machine's own `frame_copy`; and the held picture is held through two `advance`
calls, the second given 100 ms, so that a core writing its slot early is caught.*

---

## 6. What moves into `emusen-gpu`

[R, both cores read for what they duplicate; A for where each goes]

| Piece | Today | Where it goes |
|---|---|---|
| A device made by someone else | §5's seam, stated and not built | **`GpuDevice::lent`**, version 1.2: the lender's handles and a queue of the core's own; `Drop` destroys only what the crate made |
| The picture handed to the host | each core's own: Diamond's two host `slots` and `Back::Slot`, MarsRT's `Pictures` readbacks | **`PictureOut`**: the picture copied at the end of a submission into a host slot (a readback, today's) or a device slot with `ready` signalled (§5), the device slot reused only once `done` has reached it; Diamond first, MarsRT when it adopts the minor |
| Submissions that wait on and signal semaphores | none | `Commands` gains the semaphores a submission waits for and signals, for `PictureOut` |
| The GLSL-to-SPIR-V build step | `ShaderCompiler` in WiseMan's fixtures, already one; each core's check is a dozen lines over it | stays: it is a test's, and no player's build compiles a shader |
| The SPIR-V cache | Serenity's, in SQLite (`EmuSen_Serenity.md` §9.4) | stays: only Serenity compiles at run time; a core embeds its SPIR-V |
| Upscaling and resolving | MarsRT's average (the RDP's antialiasing), Diamond's scaled walk | stay: each is the console's arithmetic, its shaders the console's |

---

## 7. Diamond, and MarsRT

**Diamond** [C]. With a device lent and the host asking, the renderer copies its picture at the end of the frame's
pending submission into a device slot of `PictureOut` instead of a host slot, signals `ready`, and hands the host
**this frame's** picture; the host's chain waits for it on the device. The machine still never waits: the next frame's
submission waits for this one before it records, as in §38.1 of `Diamond_Enhancements.md`. So the frame of latency
that section pays goes, and with it nothing else changes: the renderer keeps nothing in the machine's state, so the
goldens and the state digests are untouched, and with no device lent the picture is the frame before, as today.

**MarsRT** is not on the v1 interface: Mistress drives it through its C# shim over the pre-stable `marsrt_*` exports
(`EmuSen_CoreAPI.md` §13). What it needs, in order: (1) its move onto `core_exports!` and the v1 adapter, which is
§13.3's step for it and not this project's; (2) its scan-out's readback slots (`rdp/gpu/mod.rs`'s `Pictures`) replaced
by `PictureOut`; (3) the bit claimed. Its rasteriser is on `emusen-gpu` already, so (2) and (3) are small once (1) is
done.

---

## 8. The steps, and what each must show

| Step | What | Its gate |
|---|---|---|
| (a) | `emusen-gpu` 1.2: `lent`, `PictureOut`, semaphores in submissions | the crate's tests, a lent device's included, clean under the validation layer; MarsRT's suite unchanged |
| (b) | The filters, family by family: the handheld LCDs; Lottes, Scanlines and Simple CRT; the CRT's three tiers | §3.3's comparison within its tolerance on real frames, every family; then the text converted and proven by preprocessed equality; `ShaderNamesTests` over the GLSL |
| (c) | The hand-over: the chain in flight, export, `CompositionSurfaceVisual`, the fallback | a real Avalonia window (a probe of its own under `~/.cache/emusen/probe/gpu/`): pictures read back from the compositor equal the readback path's; the validation layer clean; each fallback condition forced and taking the readback |
| (d) | The core ABI's minor 1.9 | the header, the baseline, `emusen-native`, the C# host, C20 and its skew fixture, with seeded faults each failing C20 alone |
| (e) | Diamond on it | its goldens and state digests unchanged with no device lent and with one; its picture handed the frame it was drawn; the latency in frames measured |

**Measured at each step that changes a frame's cost** (the desktop, the RX 6800 alone through `EMUSEN_GPU_DEVICE` and
`EMUSEN_SLANG_GPU_DEVICE`, against the build before it, the machine's load reported with each): the render thread's
time a frame and the UI thread's where the chain moves there, and the GPU's from timestamps, at 1080p and at 4K, for
no filter, the built-in CRT at its three tiers, CRT (Lottes), and `crt-royale` and Mega Bezel SMOOTH-ADV; and for
Diamond, the latency in frames from a frame's emulation to its picture's presentation.

---

## 9. Risks

| | Risk | What tests it, or answers it |
|---|---|---|
| R1 | Avalonia's per-update copy and texture cost more than (b)'s import | §13 measures it; the choice returns to the tester with numbers if so (§4.1) |
| R2 | One vendor: RADV and radeonsi agree on an exported image's layout (§8.5.1 of `EmuSen_Serenity.md`); another vendor's GL and Vulkan may not | the UUID check, and the driver UUIDs; the tolerance test run on any second vendor before it is trusted there |
| R3 | The chain in flight brings hazards a waited chain could not have | the validation layer over every bench case, with a positive control of a dependency removed |
| R4 | Two translations of one filter differ beyond the tolerance in a way that is the translation's | the comparison per family, and a filter is not switched until it passes |
| R5 | macOS and Windows keep the readback, which costs a built-in filter there what a preset costs (§8.3 there: 0.09 against 1.8 ms) | Q2; the kit's Vulkan on macOS (K1 (a)) |
| R6 | gamescope's XWayland under Game Mode, where Mistress runs on the handheld | the tester's handheld steps (§11) |
| R7 | A host that never signals `done` stalls a core | the rule says so, the kit's C20 checks a host that does, and the reference host signals for every picture on the emulation thread |
| R8 | The lent device needs features a later core asks for and the lender did not enable | `emusen_device_lend` names the API version; a core needing more does not use the lend and says why |

---

## 10. Questions for the tester

*Decided 2026-10-15 by the tester: Q1 to Q3, each as recommended. Avalonia stays on GLX; Skia's filter path stays until a macOS build ships with MoltenVK; every picture stays on the device unless the handheld drops frames.*

| | Question | Recommended |
|---|---|---|
| **Q1** | Avalonia's X11 Vulkan mode, with Serenity's device given to it (`CustomSharedDevice`), so that one device reaches the swapchain? | Not now. GLX stays; the mode is measured as an option once (c) is built, since it changes the renderer under every window |
| **Q2** | §3.1's second stage removes Skia's filter path, which leaves a Mac, where no Vulkan device is shipped, with every built-in filter drawn plain (as the CRT is today on Skia's software path), and a Windows machine paying the readback for a built-in | Ship MoltenVK with the macOS build (`Mars_Gpu.md` §4's open publish decision), then convert and remove. Until this is answered, step (b) stops at its first stage: the SkSL text stays the one source, drawn on the device where §4's path is on and by Skia elsewhere |
| **Q3** | The device path costs the plain picture and the built-in filters 0.5 to 1 ms more CPU a frame than Skia's path did (§13.4), and saves a RetroArch preset 1.5 to 8 ms of the render thread. Keep every picture on the device, or draw the plain picture and the built-ins with Skia where it is cheaper until the kit? | Keep every picture on the device, as decided: one path, the one a core's picture on the device needs (§5), and the kit's; the cost is CPU on two threads with no wait on either. Reconsider only if the handheld's run (§11) shows frames dropped by it |

---

## 11. The handheld: the tester's steps

*(Written with the build that carries step (c), §13.)* The device path has not run on the handheld. What to measure
there, under Game Mode's gamescope and on the desktop, on the charger and on battery:

1. **The build.** From the branch `gpu-picture` at the commit that records §13 or later:
   `dotnet publish EmuSen.Mistress -c Release -r linux-x64 --self-contained true -o <folder>`, the platform library and
   the cores built into it as every publish is, and the folder copied to the handheld as earlier builds were
   (`EmuSen_Serenity.md` §8.4's `deck-out`). Nothing else changes: no setting, no file.
2. **Whether the path is taken.** Start Mistress with `EMUSEN_DEVICE_PICTURE=1` (the default) and play any game for a
   minute; the day's log in the log folder holds a `picture` warning only when a window did not take the device path,
   with the reason (§4.4). Under gamescope the compositor is XWayland's GL, as on the desktop; a warning there names
   which condition failed.
3. **What to compare**, each case once with `EMUSEN_DEVICE_PICTURE=1` and once with `=0`, a game left running for a
   minute on its title screen and then in play: the plain picture, the CRT at its three tiers, CRT (Lottes), and
   `crt-royale`, on an SNES game and an N64 game at 4×. Read Mistress's presentation readout (§2.5 of
   `EmuSen_Serenity.md`: frames offered and shown, and the draw's milliseconds) and the emulated frame rate, and note
   whether the picture is the same in both runs by eye (a photograph of each is enough to say).
4. **What would decide against it there**: a picture that differs between the two runs, a shown rate below the
   offered one with the device path and not without it, or any frame the emulation drops with it and not without.
   Report those with the log; do not change a setting to work around them.

---

## 12. Step (b), first stage: the built-in filters translated and matched (2026-10-15)

### 12.1 What was built

**`BuiltInPreset`** (`EmuSen.Serenity/Shaders/BuiltInPreset.cs`) writes a built-in filter, for a console and its
structural values, as a slang preset in a folder named by a digest of its content under `home/Shaders/builtin/`
(`DataStore.Shaders`), each pass a `.slang` file. The translation is §3.1's first stage and §3.2's table: SkSL's type
names replaced by GLSL's (eleven word patterns), each `x.eval(` by `x_eval(`, the `uniform shader` and `uniform`
lines lifted into a sampler each and one uniform block, the SkSL `main` called by a GLSL `main` with
`gl_FragCoord.xy`, an 8-bit pass written as `vec4(rgb, 1.0)` and a float pass as it is. A child read nearest is
fetched at its texel, a child read linearly sampled through the bound sampler, the preset's `filter_linear` set for
every image some pass reads linearly. `RowsToALine`'s reducing pass is a first pass at a source scale of 1 / *n*;
the NES's raw frame is a lookup image named `raw`, black until replaced. **The plain picture and the two simple
effects** (`BuiltInShaders.AsFilter`) are one-pass filters whose `image` is the frame stretched over the rectangle,
nearest or linear as `BilinearFiltering` says, rows once, as `GameFrameControl`'s local matrix draws them.

**One thing was added to the chain**, in both implementations: a lookup image replaced in place at the same size
(`SlangChain.ReplaceLut`, the library's `load_lut`), so that the raw frame costs no image a frame (§3.4).

### 12.2 How they were compared

`DeviceFilterMatchTests` draws each filter both ways on the RX 6800: Skia from its SkSL on a GL context through
`GameFrameControl` and `ShaderBench.Picture`, as every device test of `EmuSen_Serenity.md` §3.9 does; and the chain
from the translation, advanced with each frame and rendered once for it, as the runner renders. The window is the
picture's own rectangle at a whole-pixel position, so that §3.3's fractional placement does not enter. **The frames**
are four synthetic patterns (a gradient under a moving checker, every level and edge different) followed by each real
game's picture of that size, from the frontends' resume screenshots in the ROM library, copied into scratch and never
committed: five Game Boy and Game Boy Color pictures, three NES, three SNES, two Genesis, and four N64 (two NTSC at
640 × 480 and two PAL at 640 × 576, with their progressive fields of 240 and 288 rows taken from them). The variable
`EMUSEN_GPU_FRAMES` names the folder; without it the synthetic frames alone run. Each picture is compared channel by
channel.

**A difference at a tie.** A channel that differs by more than one level is drawn again by the chain with every
pass's sampling point moved by 1/1024 and then 4/1024 of a pixel, in the eight directions; if Skia's value lies
within one of the range those drawings give, the channel sits where the filter meets a discontinuity (a nearest
texel's edge, a mask's stripe boundary, a `floor`) or a slope too steep to resolve at that distance, at its pixel's
centre. Such a channel is counted as a tie. A drawing that changes when its sampling points move by four thousandths
of a pixel samples a boundary there, and which side either path takes is not a property of the filter.

### 12.3 What was measured

Over 474 pictures (each filter family's variants, consoles, frames and window sizes; `DeviceFilterMatchTests`, 22
cases, 355 with the Serenity, Slang, ScreenFilter and GameFrameControl classes run beside them, all passing):

| Family | Pictures | Channels differing at all, most in a picture | Beyond one | Of those at ties | The rest |
|---|---|---|---|---|---|
| The six handheld LCDs, at 2× and 5× | 72 | 0.22% | 0 | — | 0 |
| The plain picture and the simple effects, bilinear | 84 | 0.10% | 1 | 1 | 0 |
| The same, nearest | 84 | 6.8% | 2,853,367 | all | 0 |
| CRT (Lottes) | 29 | 5.3% | 3,074,905 | all | 0 |
| CRT, every tier and signal, nine kinds of frame of five consoles | 114 | 0.52% | 973 | 962 | 11, at most 2 in a picture |

Every channel in the first two rows is within one level: the translation computes what the SkSL computes, in a
different compiler's order.

**The nearest-sampled rows and Lottes**, refuted §3.3's prediction that ties would be rare. They are frequent because
Skia's `coord` is not the pixel's exact centre: it is a varying interpolated across the drawn rectangle, which a probe
showed by drawing `fract` of a large coordinate both ways, and at a scale that is not whole a column or row of
pixel centres falls exactly on a texel's edge every few texels (every fourth texel at 2.25×). Skia's interpolated
centre lands a few millionths below the edge and takes the texel before; the chain's exact centre takes the texel at
the edge. The same holds for Lottes's aperture mask, whose stripe test `fract(x / 6) < 0.333` meets a boundary at
every sixth pixel along a diagonal: for coordinates above about 512, Skia's interpolation error moves those pixels
into the blue stripe where the chain's exact centres leave them in the red. **The chain draws the intended pattern**
(two pixels of each colour), and Skia's an unbalanced one (one red, two green, three blue), which is a defect of the
SkSL path that the translation does not have. Every one of these channels is a tie by §12.2.

**The CRT's eleven** (in 114 pictures of 4,665,600 channels: at most 2 in a picture, 2 to 14 levels) are not ties: the
nudged drawings do not reach Skia's value. Most are 2 levels in dark parts of composite and S-Video pictures, where
the receiver's passes subtract nearly equal half-float signals; the largest, 14 levels, is one pixel of an
*EarthBound* picture under composite at Balanced and Accurate. Their cause was not isolated. An experiment that drew
the float passes at 32 bits on both sides to remove the half-float rounding was confounded: Skia does not render
`RgbaF32` surfaces on the device, drew those passes in memory, and the two paths then differed in a million channels;
it is recorded as a negative result. **The tolerance is therefore stated as measured**: every channel within one
level; beyond one, a tie by §12.2; and of the rest no more than one channel in a million, which the CRT's worst
picture (2 in 4,665,600) keeps within. The bound is empirical and is labelled so.

**A mutant**: the translation reading `history2` for `history1` failed five of the six handheld LCDs by up to 84
levels; the sixth, the AGS-101, has no response blend and does not read its history, and passed, as it should.

### 12.4 What is not done in this stage

The runner does not yet draw the built-ins through the chain in the frontends: that is §13, where the device path
needs them, and §3.1's second stage (the text converted and the Skia path removed) waits on Q2. The NES's raw frame
reaches the chain's lookup image in the runner of §13; the comparison above drew the NES under the CRT from its
palette's colours, not from the raw frame, and the PlayStation's pictures were synthetic only (Diamond's bench discs
were not run for this step).

---

## 13. Step (c): the hand-over to the compositor, as built (2026-10-15)

### 13.1 What was built

**On the device** (the platform library, interface 11): `Shown`, an image exported with its two binary semaphores
(`emusen_serenity_shown_new`, `_free`), its memory not dedicated, as Avalonia's GL import takes it, laid out linearly
or in the device's tiling; the device's UUID and whether it exports (`emusen_serenity_device_identity`), the device
enabling `VK_KHR_external_memory_fd` and `VK_KHR_external_semaphore_fd` where offered; and `chain_present`, which runs
the passes for a view no larger than the target and copies the last pass into the target's corner, taking the image
back from the external queue family with its contents discarded, giving it back in the transfer-source layout Avalonia
waits for it in, waiting on the device for `free` and signalling `ready`, and returning **without waiting**. The device
keeps one submission in flight: the next call that writes what a pass reads (a frame's staging, a pass's uniforms and
descriptors) first waits for the last one's fence, which at 60 frames a second has long signalled. **A frame's upload
is now recorded into the submission that runs its passes**, the readback's and the present's alike, so a frame costs
one submission where it cost two and the host waited for both.

**In Serenity**: `SlangShown`, `SlangChain.Present`, `SlangVulkan.Identity`; `DevicePresenter`, three slots of a
`Shown` imported once each with `ImportImage` and `ImportSemaphore`, shown by a `CompositionSurfaceVisual` inside a
container visual that clips it, updated with `UpdateWithSemaphoresAsync`, a slot reused only once its last update's
task has completed and a frame skipped when none has; and `GameFrameControl`'s device path (§4.3, §4.5): on its
`Render`, on the UI thread, the newest offer is run through the preset, the built-in filter or the plain picture as a
`SlangRunner` over its translated preset (§12), presented, and placed at the destination of §2.9 and §2.10 of
`EmuSen_Serenity.md`; the control then issues only a draw that keeps the display's clock. `EMUSEN_DEVICE_PICTURE=0`
keeps Skia's path. Mistress logs the reason when a window does not take the device path.

### 13.2 Two things the probe found, and what was done

**The layout.** Imported optimally tiled, the image arrived in the compositor scrambled into 64-pixel columns, dedicated
memory or not: radeonsi lays out a memory object imported without `GL_DEDICATED_MEMORY_OBJECT_EXT`, which Avalonia
does not set, by its own rule, and reads it as linear [M, the probe's snapshot against the chain's readback, 2.33
million channels differing]. Laid out linearly, the picture arrived exact. Since another vendor's driver may read the
other way, **the presenter finds out**: on creation it presents a 13 × 7 picture each way and reads it back from the
compositor (`CreateCompositionVisualSnapshot`), and keeps the first layout that the compositor shows as written, or
refuses. On this machine it chooses linear under GLX and under Avalonia's Vulkan mode alike.

**The row pitch.** Linear at an odd width (1,403 pixels), the picture arrived sheared: the two drivers pad a linear
row differently. The image's rows are now padded to a whole number of 64 pixels, the picture copied into its corner,
and the visual sized to the padded image with the container clipping the padding; at 1,403 the compositor's picture
then equals the chain's own render, every channel [M]. The self-test's odd width checks the padding as well.

### 13.3 Proof in a window

The frontends' windows on this desktop open minimized and are never drawn, which no change of the probe's could
undo, so the windows were run in a **nested, headless KWin** (`kwin_wayland --virtual --xwayland`, the probe's
`~/.cache/emusen/probe/gpu/nested/`) at 1920 × 1080 and 3840 × 2160, with Mesa's `DRI_PRIME=1` putting the windows'
GL on the RX 6800, which is a real compositor and a real Avalonia window under XWayland [M].

- **The picture.** `GameFrameControl` with CRT (Lottes) on an SNES-shaped picture: the compositor's own picture of the
  surface, read back, equals a second chain's render of the same frame, **0 channels differing**, at 1,403 × 1,052.
- **The fallbacks**, each forced: Avalonia's EGL mode (no import offered) and a Vulkan device on the integrated GPU
  (another UUID) are refused with their reasons and the picture is drawn as before; the C#'s device exports nothing;
  `DevicePresenterTests` holds each of §4.4's conditions alone.
- **The validation layer**, with synchronisation validation, its shader-access heuristic and no message limit, over
  120 frames each of the plain picture, CRT (Lottes), the CRT at Accurate, `crt-royale` and Mega Bezel SMOOTH-ADV in
  the window: **0 findings**. As a positive control, the transition of the last pass before its copy removed: 346
  findings (`VUID-vkCmdCopyImage-srcImageLayout-00128` and a read-after-write hazard), so the layer was listening.
  **Not seen by it**: the image taken back from the external queue family removed gave no finding, since the layer
  does not track an image's layout across a release to `VK_QUEUE_FAMILY_EXTERNAL`; that barrier rests on the
  specification and on the compositor's picture being right.
- **Frames.** Every frame offered was handed over (300 of 300, against the Skia path's 274 to 290 drawn); no frame
  was skipped for want of a free image.

### 13.4 What it costs

Each case 300 frames after 120 of warm-up, paced at about 15 ms, the CPU time of each thread read from `/proc`
(10 ms ticks, so ±0.03 ms a frame), two rounds with the order of the paths reversed in the second; medians. The UI
thread on the device path includes the frame's chain and its present; "render" is Avalonia's render thread, which on
the device path does the compositor's copy of the image (§1.2) and composes. "Draw" is the control's own count:
the render thread's draw on the Skia path, the UI thread's present on the device path. Load averages 2 to 10 (other
work on the machine) [M, `results-c2.txt`]:

| Picture | Window | Skia: UI + render CPU | Skia: draw | Device: UI + render CPU | Device: present |
|---|---|---|---|---|---|
| plain | 1080p | 0.27 + 0.23 = 0.50 | 0.08 | 0.52 + 0.72 = 1.23 | 0.23 |
| CRT, Performance | 1080p | 0.30 + 0.37 = 0.67 | 0.17 | 0.68 + 0.95 = 1.63 | 0.27 |
| CRT, Balanced | 1080p | 0.32 + 0.38 = 0.70 | 0.23 | 0.57 + 0.78 = 1.35 | 0.27 |
| CRT, Accurate | 1080p | 0.30 + 0.45 = 0.75 | 0.27 | 0.57 + 0.82 = 1.38 | 0.29 |
| CRT (Lottes) | 1080p | 0.28 + 0.30 = 0.58 | 0.12 | 0.50 + 0.67 = 1.17 | 0.22 |
| `crt-royale` | 1080p | 0.32 + 0.93 = 1.25 | **1.87** | 0.67 + 0.82 = 1.48 | **0.36** |
| Mega Bezel SMOOTH-ADV | 1080p | 0.35 + 1.85 = 2.20 | **4.86** | 1.32 + 0.87 = 2.18 | **0.95** |
| plain | 4K | 0.32 + 0.30 = 0.62 | 0.17 | 0.55 + 1.03 = 1.58 | 0.17 |
| CRT, Performance | 4K | 0.30 + 0.38 = 0.68 | 0.25 | 0.57 + 1.03 = 1.60 | 0.19 |
| CRT, Balanced | 4K | 0.42 + 0.53 = 0.95 | 0.44 | 0.55 + 0.87 = 1.42 | 0.19 |
| CRT, Accurate | 4K | 0.32 + 0.52 = 0.83 | 0.38 | 0.58 + 0.80 = 1.38 | 0.19 |
| CRT (Lottes) | 4K | 0.32 + 0.35 = 0.67 | 0.22 | 0.53 + 0.83 = 1.37 | 0.15 |
| `crt-royale` | 4K | 0.45 + 2.75 = 3.20 | **5.71** | 0.67 + 0.87 = 1.53 | **0.27** |
| Mega Bezel SMOOTH-ADV | 4K | 0.30 + 2.27 = 2.57 | **9.55** | 1.83 + 1.13 = 2.97 | **1.48** |

**What it says.**

- **A RetroArch preset's frame leaves the render thread.** On the Skia path its render thread waits for the chain,
  reads back and uploads: 1.9 to 9.6 ms of the thread that composes the window. On the device path the UI thread
  spends 0.3 to 1.5 ms recording and submitting, waits for nothing, and the compositor's thread spends what it spends
  on any picture. At 4K, `crt-royale` costs the two threads together half the CPU it did.
- **The plain picture and the built-in filters cost more CPU on the device path**, 0.5 to 1 ms a frame across the two
  threads: on the Skia path they were a texture upload and a shader in Skia's own pass, with no readback to remove,
  and the device path adds a submission on the UI thread and, on the render thread, Avalonia's copy of the image into
  a texture it makes for each update (§1.2). R1 is realised here; §4.1's rule for putting the choice back to the
  tester is not met, since the (b) of §8.5.1 measured 1.12 ms of the render thread for the same Lottes picture
  against the device path's 0.50 + 0.67 ms on two threads with no wait, but the trade is stated as Q3.
- **Before the upload was folded into the present** (`results-c1.txt`, the same matrix the hour before), the device
  path's UI thread spent 0.4 to 2.1 ms presenting, its frame's upload waited for on its own; folding it brought the
  present to 0.15 to 0.36 ms for every case but Mega Bezel.
- **GPU time** was not read in the window. The passes are the chain's, unchanged; the device path removes the
  readback's copy and Skia's upload (0.40 and 0.43 ms at 1080p, 1.61 and 1.66 at 4K, §8.3 of `EmuSen_Serenity.md`)
  and adds a copy of the shown picture on the device and the compositor's copy of it, of the order of §8.5.1's 0.02 to
  0.05 ms each.

### 13.5 What is not done

The handheld (§11), Windows and macOS (the fallback is chosen there by §4.4; not run), a second vendor (the layout
test covers the import's layout and pitch, not a driver's other choices), GPU timings in the window, and a mutant of
the external acquire that the layer could see. Hotaru and `FramePresenter` take the device path too, as hosts of
`GameFrameControl`; only Mistress logs why a window does not.

---

## 14. Step (a): `emusen-gpu` 1.2, as built (2026-10-15)

`GpuDevice::lent`, `Timeline`, `Commands::{wait_for, signal}`, `create_buffer_shared` and `PictureOut`, as §6 lists;
`EmuSen_Gpu.md` §3 and §5.1 are the surface and the record. Three things differ from §5 and §6 as designed [A]:

- **The handles cross as integers** with the loader's `vkGetInstanceProcAddr`, not as `ash` types (`Lent::from_raw`
  takes the plain function pointer the core ABI carries), so the crate's `ash` is not a lender's concern.
- **`PictureOut` waits only for a picture the lender was told of** (`tell`), which is §5.3's "handed" made exact.
- **`hand` takes `&self`**, its counters in cells, since Diamond records a picture's hand-over inside a recording
  that borrows its owner.

Proof [M]: 17 tests of the crate on the RX 6800, 5 of them a lent device with the test as its own lender; clean under
the validation layer with synchronisation validation; three mutants, two killed and one (exclusive sharing) surviving
on a driver that does not need the ownership transfer, recorded in `EmuSen_Gpu.md` §5.1.

## 15. Step (d): the core ABI's minor 1.9, as built (2026-10-15)

`EmuSen_CoreAPI.md` §6.29 is the normative text and §28.12 the kit's case; §17 there records the minor.

**Built** [C, as §5 designed, with the two refinements of §5.3 and §5.5 marked "as built"]:

- the header: the bit, the appended `device` (104 bytes), `emusen_device_lend` (64), `emusen_device_frame` (56) and
  `emusen_core_frame_device` with its type; the baseline regenerated, **460 facts**, `--check` and `--against` 1.8's
  baseline passing (only lines added and the minor raised);
- the guard's Rust checker, which learned the two structs and a member that is a function pointer;
- `emusen-native`: `Create.device`, filled only for a core claiming the bit and only from a whole struct;
  `Core::frame_device`; the export; the reference test core claiming the bit, keeping a lend and drawing on nothing,
  with the seeded fault `FAULT_DEVICE_PICTURE_UNLENT` (1 << 23);
- the C# host's reading of the surface (`CoreInterface`: the bit, `CreateParams.Device`, `DeviceLend`, `DeviceFrame`,
  `FrameDevice`), held to the header by `CoreAbiTests`; it lends nothing yet (§16.3);
- the kit's C20 and its own lender (`lender.rs`, `ash` 0.38, the version `emusen-gpu` links), playing a host of 1.9.

**Proof** [M]: `emusen-native` 75 tests; the guard's 6; the kit's 30, among them each seeded fault failing its case
alone (`FAULT_DEVICE_PICTURE_UNLENT` C20 alone; `FAULT_ONLY_HOST_1_0` now C20 with C12, C17 and C18, since C20 plays a
host of 1.9); WiseMan's `CoreAbi` tests. A defect of the kit's own C20 was found by running it on Diamond: the case
played the session's host minor, 0 by default, so its create passed 88 bytes and no lend reached the core, and the
test core, which answers 0 lent or not, could not show it. C20 now plays 1.9; the Diamond run of §16 is its proof.

## 16. Step (e): Diamond on 1.9, as built (2026-10-15)

### 16.1 The core

`Diamond_Enhancements.md` §56 is the record: the renderer's slots became `PictureOut`; lent a device, the picture is
handed the frame it is drawn, `frame_copy` reading it back on demand; a seed goes up through the readback buffer; the
bit is claimed. A defect of the renderer's drop, older than this work (478 validation findings with no lend, objects
destroyed under a pending submission), was found by the validation layer and fixed: 0 findings after.

### 16.2 What was shown

- **No device lent**: Diamond's suites unchanged, the goldens and the state digests among them [M].
- **A device lent** (the kit's, C20, `animated-triangle.exe` at 2×, 200 frames): 120 of 120 pictures handed on the
  device, each equal to the machine's own `frame_copy`; the lent machine's pictures equal to the lendless machine's a
  frame later, every frame; the held picture unchanged; the other 19 cases passing. Under the validation layer with
  synchronisation validation, 0 findings over the whole run [M].
- **The latency, in frames**: one frame without a lend (§38.1 there), **none** with one, the core's part of §8's
  "from a frame's emulation to its picture's presentation" [M, the same run].
- **A mutant** of `PictureOut` with no wait for `done` fails C20 alone [M].

### 16.3 The reference host's lend, as it was planned

*Built the same day: §17 is the record.* The plan, as written before it was built:

1. **The device grows** (§2's table): Vulkan 1.2 where offered, `timelineSemaphore` enabled, a second queue (on the
   RX 6800 one of the four in the compute family, §1.3) and a timeline `done`; the platform library's interface
   answers the lend.
2. **The intake.** A device picture taken by the chain instead of pixels: the copy of the core's buffer into the
   chain's history image, recorded into the frame's submission as uploads now are (§13.1), waiting for `ready` and
   signalling `done` with the picture's value.
3. **The frames a host does not show.** The UI thread takes the newest picture and drops older ones, and a dropped
   picture must be let go of in order (§5.3); when nothing is shown at all (a minimised window, fast-forward with
   rendering skipped), `done` must still advance or the machine stalls within two frames. The form argued here [A] is
   an intake ring on the emulation thread: right after `frame_device`, a copy of the slot into one of three host-owned
   device buffers, on a third queue of the host's, waiting for `ready` and signalling `done`, so that the core is let
   go of at the device's pace whatever the window does; the chain then reads the newest ring buffer. It costs one
   copy on the device a frame (of the order of 0.02 to 0.1 ms at 4×) and keeps every rule of §5.3 inside the host.
4. **The C# path**: `CoreMachine` lending at create where the core claims the bit and the device can lend, calling
   `frame_device` after `advance`, and handing the frame hand-off a device picture; the Managed chain, which cannot
   take one, keeping `frame_copy`.
5. **Measured**: the latency in frames from a frame's emulation to its presentation, in a window, with and without
   the lend.

Each is the host's and changes no surface of §5.

## 17. Step (f): the reference host lends its device, as built (2026-10-15)

### 17.1 What was built

The plan of §16.3, in its order:

1. **The device grows** (`Platform/serenity/src/vulkan.rs`): the instance is made at Vulkan 1.2 where the loader offers
   it; the device enables `timelineSemaphore` where the card does, and asks for two more queues where the card has
   them (`lend_queues`): the core's, in a compute family of its own, and the host's intake queue beside it or on the
   graphics family; the host's graphics queue is never lent. On the RX 6800 the core is lent queue 0 of the compute
   family and the intake is its queue 1.
2. **The lend and the intake** (`lend.rs`): one lend for each core machine, its own timeline `done` (a new machine's
   values start again at 1, so `done` cannot be shared between machines) and a second timeline `taken`; a ring of
   three device buffers. `take`, on the emulation thread, picks a buffer that is neither the newest nor the one a
   chain has claimed, copies the core's picture into it on the intake queue after waiting for `ready`, and signals
   `done` with the picture's value and `taken` with the ring's count; a value not newer than the last is ignored. The
   core is thereby let go of at the device's pace whatever the window does, and §5.3's rule ("in order") is kept by
   one thread.
3. **The chain's intake** (`chain.rs`): `advance_lent` waits for the chain's last submission, lets go of the buffer it
   claimed, claims the newest and records its copy into the history image in the submission that runs the passes,
   which waits on the device for `taken`; with nothing newer, the frame is unchanged.
4. **The C# path**: `SlangLend`; `CoreMachine` takes a lend at create (only for a core that claims the bit), settles
   its copies before the machine is freed and lets go of it after, and `TakeDeviceFrame` asks `frame_device` after
   `advance` and `present` on a frame that is drawn; `CoreEngine` implements `IDevicePicture` (`DeviceLender`, `Lend`,
   `PictureOnDevice`); `CoreFactory.Loaded` and `EmulatorSession` pass the lender; `GameFrameControl.TryLend` lends the
   presenter's device; `FrameHandOff` and `GameFrameControl.UpdateFrame` carry a lend in place of pixels, which only
   the device path draws; Mistress lends at load (and in the launch screen's early load) and offers the picture on the
   device when the core says it is there and the window presents on the device, reading pixels back only for a
   rewind's picture. `EMUSEN_DEVICE_LEND=0` lends nothing.
5. **The setting's cost** in Diamond's four hints now reads "a frame later unless the frontend presents it on the
   same card".

### 17.2 What was shown

- **On the device, the frame it is drawn** [M, `DevicePictureLendTests`, the RX 6800, with and without the validation
  layer]: two Diamond machines through `CoreFactory`, one lent the slang device; with "Internal resolution" at 2x from
  frame 3, every picture the lent machine hands is taken, read by a chain on the device, and equals its own
  `frame_copy`, byte for byte, 11 of 11; each equals the lendless machine's picture a frame later; before the renderer
  is on the two are the same picture at the same frame. The layer, with synchronisation validation, said nothing.
- **In a real window** [M, the nested KWin of §13.3, a probe of its own, `GameFrameControl` with the plain picture and
  with CRT (Lottes), the lent machine at 2x]: 300 of 300 pictures offered on the device shown; the compositor's
  picture of a frame taken on the device equals the compositor's picture of the same frame offered as pixels, **0
  channels differing** at 1,280 × 960, for both; under the validation layer with synchronisation validation and its
  heuristic, **0 findings** over each run.
- **A positive control**: the intake's wait for the core's `ready` removed, the same window run gave 181
  `SYNC-HAZARD-READ-RACING-WRITE` findings and its picture differed in 447,423 channels; the lend's code was then
  restored, byte for byte.
- **The emulation side's cost** [M, the same probe, 300 frames, load 1 to 4]: the machine's frame, its picture's
  hand-over and the offer took **0.79 to 0.89 ms** lent against **1.33 to 1.57 ms** for the lendless machine with its
  readback and copy of 640 × 480 pixels, a saving of 0.5 to 0.7 ms a frame on the emulation thread; the UI thread's
  present was unchanged (0.17 to 0.22 ms).

- **A defect found on the way**: `SlangVulkan.TryCreate` with the library chosen explicitly (as tests choose it)
  called the library's first export before the library was attached when nothing in the process had attached it yet,
  a null function pointer; the test host crashed (exit 139) whenever such a test ran first. It now attaches first and
  refuses with the library's report when it cannot.

### 17.3 The latency, in frames

From a frame's emulation to its picture's presentation, Diamond's enhanced picture was **one frame** later than the
console's own picture path (§38.1 there); lent the device, it is **none**: the picture handed after `advance` N is
frame N's (C20 and `DevicePictureLendTests`), the host takes it on the device in the same frame, and every later stage
(the hand-off, the UI thread's present, the compositor) is the one every core's picture already took. The window's
own frames of presentation are unchanged by this and were not measured again.

### 17.4 What is left

The handheld (§11; its device lends where its card offers the queues and timeline semaphores); Hotaru, which hosts
`GameFrameControl` but does not pass a lender; MarsRT (§7); a lend refused by a core's device (a version below 1.2)
falls back to the core's own device silently, the core's note not saying so.

## 18. The handheld's black screen (2026-10-15): a GLX drawable destroyed under a present, kept by Mesa's own option

### 18.1 The fault

Reported by the tester at about 20:10: on the handheld (SteamOS 3.9.2, Ryzen Z1 Extreme, RADV PHOENIX on Mesa
26.2.0-devel, Avalonia on X11/GLX under gamescope's Xwayland), with WiseMan 370c78a5, every PlayStation game loaded to a
black screen. Mistress's own line showed the emulation at a steady 60 frames a second, "offered 60.8, shown 6.0" for the
first second and "shown 0" every second after. The handheld's error log held two reports of an unobserved task: a
superseded image's imports let go of off the UI thread, which Avalonia refuses (§18.3).

### 18.2 What was measured

A probe of the presentation path without Mistress (`~/devpicture/` on the handheld): `GameFrameControl` in a window,
Diamond on a thread of its own handing its pictures as Mistress does, the frames shown counted each second, run under a
headless gamescope (`gamescope --backend headless`) and under a nested headless KWin, through `systemd-run --user`.

| Where | Path | Shown a second |
|---|---|---|
| the handheld, gamescope | the device path, pixels, nothing lent | a few in the first second, then **0** |
| the handheld, nested KWin | the same | a few, then **0** |
| the handheld, gamescope | the device path off (Skia) | **60 of 60** |
| the handheld, gamescope | the device path, Diamond lent | a few, then **0** |
| the desktop, nested KWin, RX 6800 | the device path, Diamond lent | 60 of 60 |
| the desktop, nested KWin, Raphael iGPU (RADV and radeonsi) | the device path, pixels | 60 of 60 |

So the fault is the device path's hand-over itself, on this machine, and neither the lend nor gamescope alone: it
froze without a lend, and under KWin. A dump of the stalled process (taken by the runtime's own `createdump` at a
deliberate fail-fast, read with `dotnet-dump`) puts Avalonia's render thread in `GlxInterface.SwapBuffers`, under
`ServerCompositionTarget.Render`; the UI thread runs on (the probe's timer kept ticking), and the emulation thread too.
The frame the render thread was swapping never completed, so nothing after it was drawn, Skia's picture included.

Two facts were established by the probe's own hooks, since removed:
- **The presenter's Vulkan side completes.** Each present's submission was waited for on the processor before the
  compositor was handed the image (0.8 to 5.8 ms), and the update after it still never completed; and once the window had
  stalled, an empty submission on the slang device's queue finished within a second.
- **The fallback detects it and cannot undo it** (§18.3).

**Negative results**, recorded and not offered as an explanation: Mesa's GL thread off (`mesa_glthread=false`), radeonsi's
user queues off (`AMD_DEBUG=nouserq`), the window at 320 × 240, 640 × 480 and 1280 × 800, one slot instead of three, the
`ready` and the `free` semaphore signalled again from Vulkan 0.7 s into a stall: the picture froze in each, after 1 to 5
updates.

*At the time, the cause was open; it is §18.5, established from the sources and confirmed on the handheld.*

### 18.3 What was done first (1279c935, the hot fix)

*Superseded in part by §18.6: the device path is on by default again.*

- **The device path is off by default** (`GameFrameControl.DevicePictureAllowed`): the picture is drawn by Skia from
  the readback as before 7e7784c6, and nothing is lent to a core, unless `EMUSEN_DEVICE_PICTURE=1`; Mistress logs once
  why. A check for gamescope alone would not have been enough, the freeze being the same under KWin on the handheld.
- **A watchdog** (`WatchDevicePath`): while the device path is in use, 30 offers or more with nothing shown for a second
  give it up for the readback, the presenter let go of, said once (`picture: The picture is read back from now on: ...`).
  On the desktop's healthy device path it never fired (10 s of the lent Diamond, 60 of 60 shown). **On the handheld it
  fired after a second and did not bring the picture back**: with the render thread stuck in its swap, Skia's frames are
  not drawn either. It keeps a player from staying on the device path where the picture stops for another reason, and is
  no remedy for this one.
- **The superseded images' imports are let go of on the UI thread** (`DevicePresenter.Resize`), which Avalonia requires;
  before, they were let go of on a pool thread, threw, and left their imports and their Vulkan images behind.

### 18.4 Proven at the hot fix

- The handheld under gamescope, the default build: **60 of 60** shown each second (seconds 2 to 6; 53 of 54 in the first,
  during start-up), Diamond at 2× (`libdiamond.so` built from this branch, the core the handheld runs), and the log's one line saying the device path is
  off.
- The same with `EMUSEN_DEVICE_PICTURE=1`: the watchdog's line after one second, the picture still frozen (above).
- The desktop: WiseMan's frame-control, presenter, hand-off, lend, screen-filter and main-window tests, 233 of 233.

### 18.5 The cause, from the sources

Read from the sources of the path that stalls, matched by build-id to the dump; nothing below rests on a setting tried
to see what changes.

**The versions.** The handheld's GL is a different Mesa from its Vulkan: GL is `libgallium-26.1.2.so`, SteamOS's tag
`radeonsi-26.1.2` (18b3910ac08e: upstream 26.1.2 with SteamOS's frame-limiter patch; pacman
`mesa 26.1.2.221562.radeonsi_26.1.2-2.1`); Vulkan is RADV 26.2.0-devel. The desktop's Mesa is 26.2.4. The sources read
were gamescope (its WSI layer and Xwayland present handling), Xwayland 24.1.13, Mesa 26.1.2 and the SteamOS tag, and
Avalonia 12.1.0.

**The chain, file by file** (Avalonia 12.1.0; Mesa at the SteamOS tag unless said; Xwayland 24.1.13):

1. Avalonia uses one GLX context for its pbuffer and for the window (`GlSkiaGpu.cs:108`, `GlxDisplay.cs:127-129`). Each
   frame binds the window, swaps and makes nothing current again (`GlxGlPlatformSurface.cs:50-54, 104`;
   `GlxContext.cs:41-45, 146-147`).
2. Each `UpdateWithSemaphoresAsync` becomes a server job run before the next frame (`CompositionDrawingSurface.cs:44`,
   `ExecuteServerJobs`); the job makes the context current (`ServerCompositionDrawingSurface.cs:74`,
   `GlSkiaImportedImage.cs:182`), which with nothing current binds the **pbuffer** (`GlxContext.cs:86, 112-119`).
3. Binding the pbuffer releases the window's drawable in Mesa: `dri_bind_context` calls `driReleaseDrawables`
   (`dri_common.c:747-750`); its count reaching 0 and `keep_native_window_glx_drawable` being false, the drawable is
   destroyed, `loader_dri3_drawable_fini`, while its last present is still in flight (`dri_common.c:415-421`;
   `dri3_glx.c:147-151`).
4. The next frame makes a new drawable for the same window (`dri_common.c:345`; `dri3_glx.c:194`), with a new event id
   and `send_sbc` at 0. The X server sends `CompleteNotify` to every event selection on the window
   (`present_event.c:168-172`), and the new drawable accepts any serial up to its `send_sbc`
   (`loader_dri3_helper.c:552`), the old drawable's included.
5. `loader_dri3_swap_buffers_msc` increments `send_sbc` before it waits (`loader_dri3_helper.c:1095` in upstream 26.1.2),
   then, with a swap interval, waits `while (draw->recv_sbc + 1 != draw->send_sbc)` (upstream 26.1.2 `:1117`; the tag's
   `:1154-1156`). A stale completion has set `recv_sbc` to the value `send_sbc` reaches, so the condition never fails:
   the render thread waits in `poll` under `xcb_wait_for_special_event`, `dri3_wait_for_event_locked`, for ever.
6. The swap interval is 1 here: SteamOS's patch to this function reads `GAMESCOPE_LIMITER_FILE` and forces an interval
   of 1 when it reads 1, and under KWin the default is 1.
7. The skia path runs no server job between frames, so it keeps one drawable for the whole run, and never meets this.

**Upstream**: a known bug, introduced by a6bf07e7c245 and fixed by 0a0ee06c527f ("dri3: Increment draw->send_sbc after
waiting for last presentation", MR 42758; Mesa issues 15765 and 15802), in 26.2.0 and, on the 26.1 branch, in 26.1.5
(c11395053762). The desktop's 26.2.4 has the fix, which is why the desktop never froze.

**Ruled out by reading**: gamescope's WSI layer hooks no memory or semaphore export and no queue submission; our
binary semaphores keep Vulkan's rules (`ready` signalled before the update is asked for, `free` waited for only after
its update completed).

**In the dump**: the stalled drawable (0x7f027c0159e0, window 0x400010) reads `send_sbc` 2, `recv_sbc` 2, swap interval
1; a dead second drawable of the same window (0x7f027c0212b0, event id 0x60000e) reads `send_sbc` 1, `recv_sbc` 0.

**Confirmed on the handheld with gdb** (under a nested headless KWin, the device path on, breakpoints on
`glXMakeContextCurrent`, `loader_dri3_drawable_init` and `loader_dri3_drawable_fini`): on each applied update the
sequence is MCC pbuffer (0x600008) → FINI window (0x400010) → MCC window → INIT window, a new event id each time; the
last FINI before the freeze reads `send_sbc` 2, `recv_sbc` 1. With `keep_native_window_glx_drawable=true` in the
environment: one INIT, no FINI, and 60 of 60 shown.

### 18.6 The fix

**`DevicePictureEnvironment.Prepare`** sets `keep_native_window_glx_drawable=true` in the process's native environment,
where Mesa reads it (`xmlconfig.c:422-431`, read at `glxext.c:845-849`), unless the player has set it; it is called
first in Mistress's and the shell's `Main`, before Avalonia makes its GLX screen. .NET's `Environment.SetEnvironmentVariable`
does not change the native environment on Linux, so it is libc's `setenv`. The publish's launcher (`launcher.sh`)
exports the same, also without overriding. With it, Mesa keeps the window's drawable across the pbuffer binds, and no
second drawable meets a stale completion. **The device path is on by default again**; `EMUSEN_DEVICE_PICTURE=0` keeps
Skia's path; the watchdog of §18.3 stays.

The fix does not depend on Mesa's version: it removes the drawable's destruction, which the bug needs, rather than
deciding by `GL_VERSION`. Mesa from 26.2.0 or 26.1.5 has the fix upstream as well.

### 18.7 Proven

- **The setting by the call alone**: the probe calling `Prepare` with the variable not set and no launcher, under a
  headless gamescope, Diamond lent at 2×: the native environment read back `true`, 60 of 60 each second.
- **Ten minutes** under a headless gamescope, the default build, Diamond lent at 2×: **36,006 of 36,006** frames shown,
  never fewer than 59 a second, on the device throughout, the watchdog silent.
- **The cost** (the probe's draw time a frame and the process's CPU a second, seconds 6 to 30 of each run, Diamond at
  2×):

  | Machine, path | `true` | `false` |
  |---|---|---|
  | desktop, Skia | 0.100 ms, 87 ms/s | 0.096 ms, 86 ms/s |
  | handheld, Skia | 0.184 ms, 177 ms/s | 0.182 ms, 184 ms/s |
  | desktop, device path | 0.076 ms, **81 ms/s** | 0.087 ms, **110 ms/s** |

  None measurable on Skia's path; on the desktop's device path the setting saves CPU, the drawable no longer made
  again at every update.
- **The desktop's proofs again**, the device path on by default: the compositor's picture of a frame taken on the device
  equals the same frame as pixels, 0 channels, plain and CRT (Lottes), 300 of 300 shown; 0 validation findings over
  each; WiseMan's frame-control, presenter, hand-off, lend, screen-filter, main-window and shell tests, 270 of 270.

### 18.8 The handheld's screen during these runs

The headless test compositors run on the handheld (a nested `gamescope --backend headless` and a nested
`kwin_wayland --virtual`, each under `systemd-run --user`, from about 20:24 to 21:56) coincided with rapid flicker on
the TV attached to it. The cause is not proven: the GPU's load and the nested gamescope's input emulation (libei) and
PipeWire stream are candidates; the units had no display of the session in their environment. Runs on the handheld are
agreed with the tester beforehand from now on.

