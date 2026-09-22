# EmuSen.Serenity — the shared presentation layer

How a core's finished frame gets on screen. Core-agnostic: nothing in this project has heard of `ICore`, `VenusCore` or any console — it takes a byte array and two dimensions and draws it.

| Piece | Lives in | Knows about |
|---|---|---|
| `GameFrameControl` | `EmuSen.Serenity/GameFrameControl.cs` | Avalonia, Skia, and a pixel buffer |
| `ShaderEffect` / `FramePresenter` | `EmuSen.Serenity/FramePresenter.cs` | a `Window` and the control above |
| `BuiltInShaders` | `EmuSen.Serenity/Shaders/BuiltInShaders.cs` | nothing — two SkSL strings |
| `GraphicsSettings` | `EmuSen.Serenity/GraphicsSettings.cs` | `Galaxia.Models.GraphicsConfig` |

---

## 1. What Serenity is

The whole contract is one method:

```csharp
void UpdateFrame(byte[] rgba, int width, int height)
```

Plain RGBA8888 data plus the dimensions needed to interpret it. The frontend pulls (`core.GetFrameBufferRgba()`) and hands over bytes; Serenity presents them. `EmuSen.Serenity.csproj` references **only `EmuSen.Galaxia`**, for `GraphicsConfig`.

This is the video half of the Serenity/Endymion pair — `EmuSen_Audio_Sync.md` §7 is the audio half, written against this project as its model, and has the side-by-side table of the two shapes. `width` is taken per-frame rather than at construction because it genuinely changes frame to frame (SNES pseudo-hi-res is 512 wide); §7.2 there explains why sample rate is the audio equivalent.

Both frontends are real consumers: `EmuSen.Hotaru` and `EmuSen.Mistress` each host `GameFrameControl` directly in their own window (Mistress replacing an older CPU-side `WriteableBitmap` path). Neither uses `FramePresenter` — see §4.

The leaf rule is pinned by `EmuSen.WiseMan/Common/LeafAssemblyTests.cs`, which asserts the compiled reference set is exactly `{ EmuSen.Galaxia }`. Note what that test can and cannot catch: `Assembly.GetReferencedAssemblies()` reports references the compiler kept, so it catches a leaf *using* a core type, not an unused `ProjectReference` left in a `.csproj`.

---

## 2. `GameFrameControl` — the draw path

An Avalonia `Control`. `UpdateFrame` stores the buffer and calls `InvalidateVisual()`; the actual drawing happens later, on Avalonia's own render pass, in `Render()`.

`Render()` issues a `context.Custom(...)` with an `ICustomDrawOperation`, which is the documented way to get direct `SKCanvas` access inside an Avalonia render pass. Inside it, `context.TryGetFeature<ISkiaSharpApiLeaseFeature>()` leases the real canvas — a null feature means a non-Skia backend is in use, and the operation draws nothing rather than failing.

Which real GPU API sits under Skia's `GRContext` is decided by Avalonia's platform backend, not by anything here. `EmuSen_Project_Overview_v2.md` §2a has the per-platform breakdown (ANGLE→D3D11 on Windows, Metal on macOS, X11 on Linux) and the reason `.UseX11()` is what both `Program.cs` files call.

### 2.1 Letterboxing, and the reference canvas that was deliberately not ported

`ComputeLetterboxRect(sourceWidth, sourceHeight, actualWidth, actualHeight)` scales the source as large as it fits inside the destination, centered. It is a `static` method taking four doubles and returning a tuple — no Avalonia or Skia types — specifically so `EmuSen.WiseMan` can test the arithmetic without a render pass.

**It fits the frame into the control's real bounds, not into a fixed configured-resolution canvas.** The predecessor Raylib pipeline letterboxed into a `GraphicsSettings.WindowWidth`/`Height` reference canvas, and that was not an oversight to be faithfully ported — that canvas existed to give the old on-window debug overlay (VRAM sheet, CGRAM swatch, register text, all drawn at fixed pixel offsets around a small 2x-scaled game view) a fixed layout to share with the game screen. That overlay is gone, removed with the Raylib→Avalonia migration; `EmuSen_Settings_Reference.md` §3 records what replaced it and why it wasn't load-bearing.

With the overlay gone, reproducing the reference-canvas math would show the game tiny in one corner with dead space filling the rest — a visible regression dressed up as fidelity. Fitting the frame's own native aspect ratio into the control's actual bounds is what "show me the game, scaled to fit the window" now means.

### 2.2 Nothing GPU-adjacent is allocated per frame

This is the rule the draw path is built around, and it was learned twice, expensively:

- A fresh GPU-backed `SKSurface` allocated every frame as an offscreen upscale target caused a black-screen/flicker bug that reproduced on **both** X11 and Wayland and looked for a long time like a windowing problem.
- A fresh `SKRuntimeShaderBuilder` built from the cached effect and disposed every frame segfaulted the process on the *second* shader-active frame, because `SKRuntimeShaderBuilder.Dispose()` in SkiaSharp 3.119.4 corrupts the `SKRuntimeEffect` it was built from — a 100%-reproducible crash for anyone who selected Scanlines or CRT and kept playing.

`EmuSen_Project_Overview_v2.md` §2a is the full account of both, including how each was root-caused and how the fix was verified. What survives in the code:

- **`GetEffect(effect)`** compiles a built-in SkSL source on first use and caches the `SKRuntimeEffect` for the control's lifetime. SkSL compilation is expensive and the effect is frame-invariant, so lazily-once is the right cost. The `SKShader` built *from* it is not frame-invariant — it rebinds each frame's own image — so that part stays in the draw op.
- **`GetBuilder(effect)`** caches one `SKRuntimeShaderBuilder` per effect the same way. Never disposed, never reconstructed; only its `Children`/`Uniforms` entries are rebound per frame.
- **No offscreen surface on either path.** With no shader active, `DrawImage` scales straight to the destination rect. With one active, the shader's `image` child is built from `sourceImage.ToShader(..., SKMatrix.CreateScale(scaleX, scaleY))` — see §3 for why that local matrix is what makes the shader math correct.

### 2.3 `DrawOp` carries an immutable snapshot

Avalonia may call `Render()` again on the same control before a previously-issued draw operation has actually run. The draw op therefore takes the buffer, dimensions and effect as constructor arguments and never reads back through the control's mutable fields. The one thing it does hold a reference to the control for is `GetBuilder` — the cache that must outlive any single operation.

`Equals(ICustomDrawOperation?)` always returns `false`. This is a live video feed; there is no frame for which reusing the previous operation would be correct.

### 2.4 The property is `ActiveEffect`, not `Effect`

Avalonia's `Visual` base class already has an unrelated `Effect` property — a compositor bitmap effect such as blur or drop-shadow. Hiding it would be legal and confusing, so the shader selector gets its own name.

### 2.5 What the render thread's drawing costs, counted where it happens (2026-09-21)

Nothing measured presentation before this. The fps readout counts `RunFrame` (§4 and `EmuSen_Settings_Reference.md`
§4.21), `pacebench` has no render thread, and `frontbench`'s picture time was the getter's. The Mars GPU work
(`Mars_Gpu.md` §15.5) left one decision waiting on exactly this number: whether handing the device's image to the
compositor, with no readback and no copy, would be worth what it couples. So `GameFrameControl` now counts what the
render thread does with each frame.

`DrawOp.Render` times two things: **the copy**, `SKImage.FromPixelCopy`, which copies the frame out of the array the
core handed over; and **the draw**, which is the draw call and a `GRContext.Flush`. The flush is there because Skia
defers a raster image's upload to the GPU until a flush, so without it the draw would time a recorded command and the
upload would land, untimed, in Avalonia's own flush after the scene. Flushing earlier moves that cost; it does not
add to it. It also records whether the lease had a `GRContext`, which is the difference between the GPU backend and
Avalonia's software one, and the frame's size.

The sums are exchanged for zero by `TakeStatistics`, from any thread, so a frontend reading them once a second sees
each frame exactly once. Mistress prints frames *offered* (handed to the control) and frames *shown* (drawn by the
render thread) side by side with the two costs; the difference between the two rates is the hand-off dropping stale
frames, which is correct behaviour (§4), not a fault.

**What this does not measure:** the GPU's own time, the compositor's swap, or anything after the flush returns; and
on the software backend "draw" is the whole scale-and-filter on the processor, which is a different quantity from the
GPU backend's. `Each_drawn_frame_is_counted_once_with_its_size_and_backend` checks the counting on the headless
platform, which has no GPU; the GPU backend's figures exist only in a real window.

**The first reading** (2026-09-21, Mistress on this machine's RX 6800 under GLX, Super Mario 64's European release, so
50 fps, the Mars GPU setting on, antialiasing off, paced; one one-second window each, read off the screen):

| Resolution | Frame | Copy | Draw and upload | Emulated fps | Shown fps |
|---|---|---|---|---|---|
| 1× | 640×576 | 0.06 ms | 0.16 ms | 50.5 | 51.5 |
| 2× | 1280×1152 | 0.29 | 0.35 | 50.8 | 51.8 |
| 3× | 1920×1728 | 0.84 | 0.84 | 50.0 | 50.0 |
| 4× | 2560×2304 | 1.71 | 1.32 | 49.4 | 51.4 |

The cost grows with the frame's area, as a copy and an upload should, and at four is some three milliseconds of a
twenty-millisecond frame, on a thread that is not the one bounding the frame rate: every multiple ran at full speed.
Two things in the same numbers are worth more than their size suggests. **The frame is twice as tall as the picture**:
2,304 lines at four are 288 doubled, because `MarsCore.Compose` repeats each line of a progressive field and the
frontend has no way to be told to stretch instead, so half of what is copied and uploaded is a copy of the line
above. **More frames are shown than offered**, by one or two a second: the control redraws when anything else in the
window invalidates it, such as this readout's own text, and each redraw copies and uploads the frame again.

### 2.6 A redraw is not a new frame (2026-09-21)

The first reading of §2.5 showed more frames drawn than offered. The control is redrawn whenever the window needs
it, and each redraw copied the frame out of its array and uploaded it again. Now each `UpdateFrame` is a new
**version**, and the control keeps the image it made from the last version drawn: a redraw of the same version
reuses it, with neither a copy nor, since Skia keeps an image's texture for as long as the image lives, an upload.
The cache is the render thread's, under a lock, and is given back when the control leaves the window.

**A new offer is always a new version, even of the same array.** Moon and Mercury hand over their PPU's own buffer
every frame, rewritten in place, so a cache keyed on the array would show a stale picture; keyed on the offer it
cannot. What stops an unchanged picture from being offered at all is the core's frame serial, read by the frontend
(`EmuSen_Multicore.md` §14), not anything here. The counters now separate draws from copies, and Mistress's readout
shows both. `A_redraw_reuses_the_frame_and_a_new_offer_of_the_same_array_is_copied_again` fails both for a control
that always copies and for one that never recopies a new offer.

### 2.7 Rows stretched here rather than repeated in the frame (2026-09-21)

`UpdateFrame` takes how many times each row is shown (`EmuSen_Multicore.md` §15), and the control letterboxes to the
shown height, `height × rowRepeat`, and draws the image into that rectangle, which stretches the rows on the GPU. With
nearest filtering this is exactly the picture a frame with its rows repeated would draw
(`Rows_stretched_here_draw_what_rows_repeated_in_the_frame_draw`).

**With bilinear filtering, the default, it is not the same picture, and that is a visible change.** Repeated rows
come in identical pairs, so the filter blends only across the boundary between pairs; stretched rows are all
different, so it blends every row with the next. On the test's deliberately high-contrast pattern, 576 of 1,024 bytes
differ, by up to 28 levels. The stretched version is the filter applied to the picture the console made rather than
to a copy of it with its rows doubled, and it is the one Mistress now shows; it is recorded here because it is a
change in what the player sees, not only in what it costs.

---

## 3. The built-in shaders

Two single-pass effects, `Scanlines` and `Crt`, as SkSL source embedded in `BuiltInShaders`. `ShaderEffect.None` is always the fallback and is bit-identical to having no shader pipeline at all.

They are faithful ports of the original Raylib/GLSL versions, not a redesign — the port was part of moving off Raylib, and doing it as a translation kept the two changes separable. They stay embedded as C# string constants for the same reason the GLSL predecessors were: no "did the build actually copy this file" failure mode while the pipeline itself is still being proven out. `BuiltInShaders` is `internal`, an implementation detail of `GameFrameControl` rather than public API; `AssemblyInfo.cs` opens it to `EmuSen.WiseMan` alone so the SkSL source can be asserted directly.

**What changes when translating GLSL to SkSL:**

- **Sampling another shader.** SkSL's `uniform shader image` plus `image.eval(coord)` is the direct equivalent of GLSL's `sampler2D` / `texture(texture0, fragTexCoord)` pair. SkSL has no `sampler2D`.
- **`coord` is in destination pixel space, not a normalized UV.** GLSL's `fragTexCoord` arrived as 0–1; SkSL's `coord` arrives in the destination's local pixel coordinates. The scanline row check uses it directly (`floor(coord.y)`), which is exactly what makes the effect run at output resolution and stay crisp at any window size. The vignette still needs the original 0–1 space its math was written against, so it divides by `outputSize` first to recover it.
- **The `image` child is sampled in the *source* image's native pixel coordinates**, not stretched to any destination rect. This is why §2.2's local `SKMatrix.CreateScale(scaleX, scaleY)` matters: without it the row and vignette math would operate over the frame's native-resolution top-left corner of the upscaled output, shading a small patch instead of the picture. The matrix maps `coord` back onto the native source, so the effect runs in real output-pixel space with no intermediate render target.

The exact darkening factors are part of the contract, not incidental: `Scanlines` leaves even output rows at full brightness and multiplies odd rows by **0.78**; `Crt` uses **0.72** and adds a vignette of `1 - dot(centered, centered) * 0.55`. `EmuSen.WiseMan`'s `Scanlines_darkens_odd_output_rows_by_the_exact_documented_factor` renders a solid-color frame and checks that arithmetic precisely, which is what proved the local-matrix rewrite correct rather than merely different.

### 3.1 The filters a frontend can offer, by name (2026-09-21)

`ScreenFilters` is the list a settings window shows and a config stores: a name for each filter and the effect it draws, with **None** first. A frontend keeps the name, never the enum's number, so a filter added later or renamed cannot shift what an old config meant, and `ByName` answers None for any name it does not know, so a config from a newer build shows the picture unfiltered rather than failing. Today the list is the two effects above, as "Scanlines" and "Simple CRT"; it is where accurate CRT and handheld LCD filters are meant to arrive (`EmuSen_Settings_Reference.md` §4.40).

### 3.2 Multi-pass filters, and the frames they look back at (2026-09-21)

A `ScreenFilter` is a list of `FilterPass`es, each an SkSL program drawn either at the game's own size (`PassScale.Source`: a panel's colour, its response) or at the size of the rectangle the picture is shown in (`PassScale.Viewport`: a pixel grid, a CRT mask). `GameFrameControl.ActiveFilter` draws one in place of `ActiveEffect` when set. `FilterChain` runs it: every pass but the last draws into a surface of its size (a GPU surface on the lease's `GRContext` when there is one, a raster one otherwise), whose snapshot is the next pass's `source`; the last pass draws straight into the letterboxed rectangle when it is viewport-sized, and its surface is stretched there when it is not.

**What a pass can ask for**, by the names it declares, and only those are bound: `source` (the previous pass, or the frame), `original` (the frame), `history1` to `historyN` (the frames before it), and the uniforms `inputSize` (texels of `source`), `originalSize`, `outputSize` and `frameCount`. Children are sampled in their own texel coordinates, so a pass computes `coord / outputSize * inputSize` and reads `floor(t) + 0.5` for a texel's centre, which is libretro's convention restated for SkSL's pixel-space `coord`. A name no pass provides is refused when the chain is built rather than drawn black. Nearest sampling everywhere unless a pass asks for a linear `source`.

**History.** A filter reads as many earlier frames as its most demanding pass declares. When a new frame replaces the cached image, the replaced image goes into the chain's history instead of being freed, newest first, and the oldest beyond the count is disposed; a history frame not yet seen reads as the frame itself. A redraw without a new frame leaves the history as it is, so ghosting follows the game's frames and not the screen's.

**The limits SkSL sets**, from the pipeline survey of 2026-09-21: loops only with constant bounds, no array initialisers or dynamic indexing, no `uint`, bitwise operators, derivatives, `texelFetch` or `textureLod`, and no preprocessor. Everything below is written within them; a general libretro `.slang` preset cannot be, which is why that is a separate plan (§3.6).

### 3.3 Which filters a console is offered

`ScreenFilters.All` lists every choice, `NamesFor(console)` the ones that suit a console: a filter with no console list suits every one (None, Scanlines, Simple CRT), and the accurate ones name theirs. The graphics window's row shows `NamesFor`, so a Game Boy is not offered a television and a SNES not an LCD. The GBA filters name a console no core in this build has, and so appear nowhere yet; they are built and tested so that a GBA core finds them waiting.

### 3.4 CRT (Lottes)

Timothy Lottes' CRT shader, in the public domain, ported from libretro's `crt/shaders/crt-lottes.slang` at its default parameters: Gaussian scanlines (`hardScan -8`) and pixels (`hardPix -3`), a small bloom (`0.15`), the barrel warp (`0.031`, `0.041`) with black outside it, and shadow mask 3 (the stretched VGA mask, dark `0.5`, light `1.5`), in linear light with the sRGB curve both ways. The port changed nothing but spelling: `vec` to `float`, the texture fetch to `source.eval` at a texel's centre, the output-pixel mask coordinate to SkSL's `coord`, and the parameters to constants. Offered for the NES, SNES and N64.

### 3.5 Handheld LCDs

Two passes each: the panel at the game's size, then its pixel grid at the screen's. The grid fades in between one and a half and three screen pixels per game pixel (two and four for the colour stripes), because below that it can only alias.

**Game Boy, Pocket and Light.** Mercury draws four neutral greys (`Mercury_Ppu.md` §6); the panel pass takes each pixel's grey, lets it lag behind its last three frames by Harlequin's response formula (`v += (previous_k − v) · rt^k`, rt 0.33 for the DMG and 0.12 for the Pocket and Light, whose panels mostly removed ghosting), and maps the result onto SameBoy's measured four-shade palette (`Core/display.c`: DMG `C6DE8C 84A563 396139 081810`, Pocket `C2CE93 818D66 3A4C3A 07100E`, Light `7FE2C3 56B495 357862 0A1C15`), interpolating so a pixel mid-change is a colour between two shades. The grid pass draws each pixel as a dot covering 86% of its pitch, the gaps showing the switched-off panel's colour (SameBoy's fifth entry), and each dot casting a shadow offset by (0.6, 0.8) pixels onto the reflector beneath it, darker for a darker dot, at 25% for the DMG, 20% for the Pocket and 10% for the backlit Light: SameBoy's MonoLCD offset, done as a multiply rather than its `min`.

**Game Boy Color.** The panel pass blends a third of the last frame in (the panel's response), then applies Pokefan531's measured sRGB matrix in linear light (`handheld/shaders/color/gbc-color.slang`, public domain: output red `0.905 R + 0.195 G − 0.10 B`, green `0.10 R + 0.65 G + 0.25 B`, blue `0.1575 R + 0.1425 G + 0.70 B`, times 0.91, gamma 2.2 each way); since 2024 that file and `gba-color` carry the same matrix. The grid pass draws vertical red, green and blue stripes, the layout fishku's `authentic_gbc` (CC0) took from a photograph, with a dimmer row at each pixel's foot.

**Game Boy Advance, and the SP's AGS-101.** The AGB-001 uses the same matrix and a quarter-frame response, with the stripes plus a dark fourth column, the layout of mGBA's `agb001` (MPL-2.0; reimplemented here from its description, not copied). The AGS-101 uses Pokefan531's `sp101-color` matrix (`0.96 R + 0.11 G − 0.07 B`, `0.0325 R + 0.89 G + 0.0775 B`, `0.001 R − 0.03 G + 1.029 B`, times 0.935), no response blend and a lighter grid, since it is backlit.

**The grid's brightness was measured and corrected before commit.** The first stripe grid (floor 0.35, gain 1.45; GBA 0.2 and 1.8) left white at about half brightness, because the lit stripe clamped at full while the other two stayed dark, and at four screen pixels per game pixel three hard-edged stripes fall unevenly into four pixels, so the GBC's green took one and a half of them and white read green. Softer edges (one screen pixel wide) and a higher floor with less gain (GBC 0.5 and 1.25, AGB 0.45 and 1.2, AGS-101 0.6 and 1.15) put white at a mean of (193, 202, 193) on the GBC and (166, 172, 172) on the AGB-001 at 4×, read from rendered pixels of *Link's Awakening DX*.

**What the numbers are, and what they are not.** The palettes and matrices are measurements, cited to who made them. The response rates, the coverage, the shadow opacities and the grid's floor and gain are choices made to look like the panels and checked by eye against rendered frames, not measured from hardware; §3 of the handheld survey records that no primary source for the DMG's pixel gap or response time was found. SameBoy's own frame blending alternates a third and two thirds by line and frame parity; this uses a constant third.

**Tests** (`ScreenFilterRenderTests`, 12): every filter compiles; each console's list; Mercury's four greys become the four DMG shades; a darkened pixel takes frames to settle and then reaches the darkest shade; magnified, a dot's centre is its shade, a gap the reflector and a dark dot's shadow falls below and right; GBC red is the matrix's red to within three levels; magnified GBC stripes run red, green, blue; Lottes draws brighter scanline centres than gaps and black curved corners; a chain holds as many frames as it reads. Seven mutants each caught (inverted palette, no response, no shadow, no matrix, flat scanlines, no history, no stripes).

### 3.6 RetroArch's shaders, the plan after this

Downloading libretro's `slang-shaders` on request and running any `.slangp` preset needs a real shader runtime, since SkSL cannot express them (§3.2): glslang to SPIR-V (Silk.NET.Shaderc), then either Vulkan directly, beside Mars's device, or SPIRV-Cross to GLSL on Avalonia's OpenGL, with presets' passes, scale types, history, feedback, lookup textures and float and sRGB targets. Legally a download on the player's request redistributes nothing, and GPL-2.0-or-later shaders are compatible with this project's GPL-3.0 besides; about ten shaders and several lookup images state no licence and may be run from a download but never copied into source (the licence survey of 2026-09-21). ~~Not started.~~ *Carried out the same night, on Vulkan directly: §7.*

---

## 4. `FramePresenter` — the bundle nothing consumes yet

A `Window` + `GameFrameControl` pair for a consumer that needs nothing else from its window: construct it, call `Present(rgba, w, h)` from any thread, `IsOpen()` to test for a user close, `Shutdown()`/`Dispose()` to close it.

**Both frontends host `GameFrameControl` directly instead.** Each has its own real window with menus, docked panels and dashboards around the game view, so a class whose entire value is owning the window has nothing to offer them. It stays as the reference implementation of the presentation contract and the coalescing pattern below, and because a future headless-ish consumer that wants a picture and nothing else is exactly what it is for.

**The coalescing contract.** `Present` may be called from any thread — in practice a frontend's background emulation thread. It offers the frame; the newest one wins, and at most one UI-thread callback is ever outstanding:

- Only one present is ever in flight.
- A frame arriving while one is already dispatched **overwrites** the pending frame rather than queueing behind it.

The consequence is deliberate: a UI thread that cannot keep up drops intermediate frames instead of accumulating a backlog of them, and the frame actually presented may not be the one that triggered the dispatch.

**Correction, 2026-08-10: the mechanism is `EmuSen.LunaP.Threading.Latest<T>` now, and the third bullet this section used to carry was describing a bug.** It read:

> `_presentScheduled` is reset in a `finally`, *after* the frame is handed to the control — so for as long as a present is actively running, callers keep overwriting `_pendingFrame` without scheduling another.

That is what the code did, in all three places that had written this mechanism out — here, `EmuSen.Mistress`'s `MainWindow` and `EmuSen.Hotaru`'s `GameWindow`, byte-identically. **A frame submitted while the UI thread was inside `UpdateFrame` could neither schedule a callback (the flag still read 1) nor be collected by the running one (it had already taken its frame). It sat in `_pendingFrame` until the next frame displaced it.**

At 60 fps nothing could see it: the next frame arrives 16 ms later and carries the fix with it, which is why it survived being copied twice. It shows when the stream *stops* — pause the emulator, and the frame at risk is the last one drawn, which is the one somebody is about to sit and look at.

It was found by generalising the mechanism into a toolkit rather than by anything going wrong here. Clearing the flag *before* the hand-off, plus a re-check for a frame that landed between the two interlocked operations, is the fix; `LunaP.md` §22.1 carries the reasoning and the test that pins it. All three copies are deleted — the frontends and this class all call `Latest<T>.Offer` now, so the three cannot diverge again.

`CycleEffect()` steps None → Scanlines → Crt → None for quick manual A/B testing (Hotaru's F8; `EmuSen_Frontend_Driver.md` §2). The enum arithmetic is factored out as the `static` `NextEffect(current)` so `EmuSen.WiseMan` can test it without constructing a `FramePresenter`, which opens a real `Window`.

---

## 5. `GraphicsSettings`

Window size, title, vsync, target frame rate and texture filtering, mapped to and from `etc/EmuSen/graphics.json`. **Every field is documented in `EmuSen_Settings_Reference.md` §3**, including which are wired to nothing and which three were dropped with the Raylib overlay; `EmuSen_Config_Reference.md` §3.3 covers the file, the clamping and the first-run seeding.

Two facts worth knowing here rather than there. Its namespace is `EmuSen.Graphics`, not `EmuSen.Serenity` — the class moved out of `EmuSen/Settings/` without renaming. And it lives in this project rather than next to `DebugSettings`/`AudioSettings` because it configures presentation, which every frontend shares, while those two configure the core they sit beside.

---

## 6. What the tests hold

All headless, in `EmuSen.WiseMan/Serenity/`, through `HeadlessUnitTestSession` — real Avalonia render passes with no display or GPU. `TestAppBuilder.cs` includes the same `LunaTheme.axaml` the real frontends do, so a render pass never runs over untemplated controls.

- `GameFrameControlTests.cs` — `ComputeLetterboxRect` arithmetic, including the degenerate zero/negative inputs.
- `GameFrameControlRenderTests.cs` — the real render path, added specifically to close the "nothing exercises this" gap that hid the §2.2 crash. Byte-identical repeated output, 120 consecutive frames without throwing, and the exact scanline pixel math.
- `BuiltInShadersTests.cs` — asserts the SkSL source itself, via the `InternalsVisibleTo` in `AssemblyInfo.cs`.
- `FramePresenterEffectCyclingTests.cs` — `NextEffect`'s cycle, without a window.
- `Common/LeafAssemblyTests.cs` — §1's reference set.

## 7. RetroArch's shaders

The plan of §3.6, carried out in stages, each committed working. The pack is libretro's own distribution, `shaders_slang.zip` from `buildbot.libretro.com/assets/frontend/`, the file RetroArch's online updater fetches (54 MB, rebuilt nightly; the copy read here was built 2026-09-22 02:00 UTC).

### 7.1 Reading a preset (2026-09-21)

`SlangPreset.Load` reads a `.slangp` as RetroArch's `video_shader_parse.c` does. Keys are merged through `#reference` chains: the referenced file first, then the referring file's keys over it, each path resolved against **the directory of the file that wrote that key**, which is what lets a preset three folders away reference `crt-guest-advanced` and still find its lookup images. A chain deeper than sixteen is refused as a loop. Per pass: the shader, `alias`, `filter_linear`, `wrap_mode` (clamp to border by default), `scale_type` with its per-axis forms and `scale`/`scale_x`/`scale_y`, `float_framebuffer`, `srgb_framebuffer`, `mipmap_input` and `frame_count_mod`; a pass with no scale type is source 1.0, or the viewport when it is the last. `textures` names the lookup images, each with `_linear`, `_wrap_mode` and `_mipmap`. **Every other key with a number is a parameter's value**, which is how 695 of the pack's presets are nothing but a `#reference` and a few overrides.

**RetroArch tolerates an unterminated quote, and so must this.** The first run over the pack failed 37 presets. Thirty-one were values like `"../../../../crt/shaders/guest/advanced/lut/trinitron-lut.png` with no closing quote, which the first reader kept with the quote at the front and so resolved to a path that does not exist; a quoted value now runs to its closing quote, or to a comment or the line's end when there is none. Six were not presets at all but fragments, files of overrides with no shaders of their own, meant only to be referenced, which the pack test now counts apart.

**Five presets in the pack are broken upstream**, not here: `bezel/koko-aio/Presets_HiresGames_Fast/Presets_Handhelds-ng/PSP*.slangp` name `../textures/overlays/psp-e1000.jpg`, which exists one folder further up, in `koko-aio/textures/`. RetroArch cannot load their overlay either. They are reported by the test and not failed.

### 7.2 Reading a pass's source (2026-09-21)

`SlangSource.Load` expands `#include` (textual, relative to the including file, refused past thirty-two deep) and `#pragma include_optional` (skipped when absent), then splits the result at `#pragma stage vertex` and `#pragma stage fragment`: lines before the first stage go to both, lines after a stage to that one only. It reads `#pragma parameter id "description" initial minimum maximum [step]` (first declaration wins, as a shared include declares the same parameter in several passes), `#pragma name` (the pass's alias from inside the shader) and `#pragma format`. Every pragma line, and every line belonging to the other stage, is kept as a blank, so both stages have the same number of lines and a compiler's line numbers point at the right line of the expanded text.

**Measured over the whole pack** (`Every_preset_in_the_pack_reads_and_every_pass_it_names_splits`, run with `EMUSEN_SLANG_PACK` pointing at an unpacked copy): 2,652 presets and 6 fragments; 1,350 distinct passes, every one of which expands and splits; no failures; 5 missing images, the upstream defect above. The test is skipped without the variable, because the pack is not in the repository and a test must not reach the network.

**Not yet**: nothing is compiled or run. Stage 2 compiles each stage to SPIR-V and reads back what it binds.

### 7.3 Compiling a pass and reading what it binds (2026-09-21)

`SlangCompiler` hands each stage to Shaderc (glslang inside it) as Vulkan 1.1 GLSL, **unoptimised**, because the reflection below reads names and an optimiser is free to strip them. `SpirvReflection` then reads the SPIR-V directly: `OpName` and `OpMemberName` for names, `Binding` and member `Offset`, `ArrayStride` and `MatrixStride` decorations for layout, and the variables' storage classes to tell the uniform block (Uniform), the push constants (PushConstant) and the combined image samplers (UniformConstant) apart. The two stages are merged into one pass: the union of each block's members, every sampler either stage names.

**Why not SPIRV-Cross.** RetroArch and OpenEmu both reflect through it (`slang_process.c`, `ShaderPassCompiler+Reflection.swift`). What the runtime needs is three lists: a block's members with their offsets and sizes, the push constants', and the samplers with their bindings. That is a hundred and fifty lines over five opcodes and five decorations, testable byte for byte, against a second native library to ship and load. The cost is that anything SPIRV-Cross would have found and this does not look for, it will not find: sampler arrays, several uniform blocks, specialisation constants. The spec (libretro's `slang-shaders` README) allows one uniform block and one push-constant block and no arrays of samplers, so none is expected.

**The packages.** Serenity now takes `Silk.NET.Shaderc` (MIT binding; its native, Google's shaderc and glslang, Apache-2.0, 9.2 MB for linux-x64) and `Silk.NET.Vulkan` (MIT), both at 2.23.0, the versions Mars and its tests already use. Both are compatible with this project's GPL-3.0.

**Measured over the whole pack** (`Every_pass_in_the_pack_compiles_and_reflects`, with `EMUSEN_SLANG_PACK`): all **1,350** distinct passes compile in both stages and reflect, in 5.6 seconds on this machine, with no failures. `A_pass_compiles_and_its_block_members_offsets_and_textures_are_read_back` pins the std140 offsets (`MVP` 0 and 64 bytes, `OutputSize` 64, `OriginalSize` 80, a float at 96, block size 100), the push constants and the two sampler bindings.

### 7.4 Running a preset on the device (2026-09-21)

`SlangVulkan` is a Vulkan 1.1 device of its own: one graphics queue, host-mapped buffers, 2D images with an optional mip chain and two views (the whole chain, to sample; level 0, to draw into), samplers cached by filter, wrap and mipmapping, and one command buffer submitted and waited on. It prefers a discrete card, and `EMUSEN_SLANG_GPU_DEVICE` narrows the choice by name. When there is no device, `TryCreate` returns null with the reason, and the frontend is to show the picture unfiltered. It is **not Mars's device**. Sharing one would couple a presentation filter to one core's renderer, and the only thing shared is the driver, which serves both.

`SlangChain` builds one pipeline per pass: a render pass with one colour attachment cleared to black, a four-vertex strip carrying `Position` and `TexCoord`, dynamic viewport and scissor, and no blending. The descriptor set layout and push-constant range come from §7.3's reflection. Each frame has two calls:

- `Advance` uploads the core's picture into a ring of history images, with repeated rows expanded first (§2.7), because a slang pass sees the picture as the screen would.
- `Render` sizes every pass, fills every block, draws the passes in one submission, and copies the last one back.

**The semantics, as libretro's spec gives them.**

Sizes and scaling:
- Every `*Size` member is `(width, height, 1/width, 1/height)` of the texture it names.
- A pass is sized by its scale type: the previous pass times the scale, the viewport times the scale, or the absolute value.
- **The last pass is always the viewport's size**, whatever it asks, because RetroArch's Vulkan driver draws it into the swapchain. For the same reason it is 8-bit (sRGB if it says so), never float.

Textures:
- `Original` and `OriginalHistoryN` are the ring.
- `Source` is the previous pass's output, or the original for pass 0.
- `PassOutputN` and an alias name an earlier pass.
- `PassFeedbackN` and alias + `Feedback` are last frame's output of any pass. Such a pass keeps two images and swaps them after each frame.
- Lookup images are loaded by name.

Samplers:
- `Source` is sampled with its own pass's filter and wrap.
- `Original` and the history are sampled with pass 0's.
- An earlier pass's output is sampled with the settings of the pass after it. This is the rule by which a preset's `filter_linearN` describes how pass N reads its input.
- `mipmap_input` gives the previous output a mip chain, blitted after the pass that drew it.

Values:
- A parameter takes the preset's value when the preset gives one, otherwise the `#pragma parameter` initial value.
- `MVP` is the orthographic map of the unit square onto clip space.
- `FrameCount` honours `frame_count_mod`.
- A member the chain does not recognise is zero.

**Deliberately constant**, since no core here produces what they describe:
- `FrameDirection` is 1, because there is no rewind through the chain.
- `Rotation` is 0.
- `TotalSubFrames` and `CurrentSubFrame` are 1.
- `OriginalFPS` is 60.

**Why readback.** The chain ends by copying its last image into host memory, and the frontend is to draw that as an ordinary bitmap. Handing a Vulkan image to Avalonia's compositor directly would save the copy. However, it needs the compositor to be on Vulkan and to import the image, which Avalonia exposes only on some backends. The copy also keeps this runtime independent of what Avalonia is drawing with. Its cost is not yet measured inside Mistress. On the test device, ten frames of `crt-royale` (12 passes, 256×224 to 1024×896, readback included) took 24 ms, about 2.4 ms a frame.

**Tests** (`SlangChainTests`, 12, on the RX 6800 by default):
- An identity pass returns the picture byte for byte.
- Repeated rows arrive expanded.
- A second pass reads the first by alias and by number, and the original.
- A pass reading its own feedback adds 10 a frame and reaches 30 after three.
- `OriginalHistory1` and `OriginalHistory2` are the frames one and two back, over four frames.
- A preset's parameter overrides the pragma's.
- Sizes and a 3× source-scaled pass report 12, 6 and 20 as they should.
- A two-pixel lookup image is sampled by name.
- `crt-royale`, `crt-guest-advanced`, `crt-lottes` and `lcd-grid-v2` from the pack build, run ten frames and draw something. This case is gated by `EMUSEN_SLANG_PACK`.

**Mutants.** Seven were made:
- history stepping forward
- no feedback swap
- no parameter override
- no row expansion
- no alias
- no source scaling
- `OutputSize` reporting the viewport

The history mutant **survived the first history test**: with two history slots, stepping the ring forward or backward lands on the same slot. The test now reads `OriginalHistory2`, and all seven are caught.

**The validation layer** (`VK_LAYER_KHRONOS_validation` with synchronisation validation and `VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1`) reports nothing over all twelve tests, the pack presets included. As a positive control, a mutant that skipped the final image's transition back to shader-read was flagged four times (`VUID-vkCmdDraw-None-09600`), so the silence is evidence.

**What this does not show.** The pack presets are checked for running and drawing, not for drawing *correctly*. No output here has been compared against RetroArch's for the same frame. That comparison would need RetroArch run headless on a captured frame, and it is the obvious next evidence if a preset looks wrong.

### 7.5 Drawing a preset in the frame control (2026-09-21)

`GameFrameControl.ActiveSlangPreset` takes a preset's path and draws it instead of `ActiveFilter` and `ActiveEffect`. Setting it creates a `SlangRunner`, which builds the chain on a pool thread; `crt-royale` takes 1.7 s to compile, and a render thread that waited for it would freeze the window for that long. **Until the chain is built, the picture is drawn plain**, as it is when the chain cannot be built. In that case `SlangFailed` is raised once, off the UI thread, with the preset's name and the compiler's or device's first line, and Mistress puts it in the status bar. When the build finishes, the control is asked to redraw, so the filtered picture appears without waiting for the next frame.

**The device is one per process**, created on first use and never closed. Every preset the player tries would otherwise open, and after a switch close, another Vulkan device. Its command buffer is now guarded by a lock, because a chain can be building (uploading its lookup images) on one thread while another chain draws on the render thread.

**Per draw**, under the control's lock:
- A new frame is advanced into the chain.
- The chain is rendered at the letterbox rectangle's size **in device pixels**: the canvas's scale times the rectangle, so a 2× display gets a 2× viewport, as RetroArch's would.
- The result is drawn as an opaque image, with nearest sampling since it is already at the size shown.

A redraw with no new frame and no change of size reuses the last image. This is §2.6's distinction between a redraw and a frame, kept so that a feedback or history preset does not advance on a repaint.

**Tests** (`SlangFrameControlTests`, through the real headless render pass):
- An inverting preset turns an (10, 20, 30) frame into (245, 235, 225) once built.
- A preset whose shader does not compile raises `SlangFailed` with its name and draws the frame unchanged.
