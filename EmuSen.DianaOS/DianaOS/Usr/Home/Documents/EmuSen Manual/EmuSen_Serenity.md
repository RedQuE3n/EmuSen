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

*Since 2026-09-21 and 2026-09-23 it also takes how many times each row is shown (§2.7) and who takes the array back (§2.8); both are optional.*

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


### 2.8 A lent array, given back once (2026-09-23)

`UpdateFrame` takes a fifth argument, `release`. A frontend whose core lends its pictures (`EmuSen_Multicore.md` §16)
passes the core's `ReturnFrameBuffer`, and the control calls it **once per offer, with that offer's array, when
nothing in the control can read the array again**. A frontend that passes nothing, which is Hotaru, `FramePresenter`
and every test that predates this, sees no change. The reason is MarsRT's picture: until this section it was a new
array per frame, 1.2 MB at one multiple and 19 MB at four, and on the handheld the collections those arrays caused
cost more than the copies did (`Mars_Native.md` §6.13).

**The claim.** An array the control has been given is never handed back while a draw operation could still copy
it, and a picture once drawn is never replaced on screen by an older one. The second is a condition of the first, and
it was not true before.

**The defect fixed on the way.** The render thread decided whether to copy by `_cachedVersion != _version`: a draw
operation holding an *older* version than the cached one satisfied the test and copied its older array over the newer
image, so the picture went back a frame. `A_draw_operation_rendered_late_never_takes_the_picture_backwards` holds two
operations, draws the newer and then the older, and on the old rule the second draw showed the older frame's red where
the newer's green had been; on the new rule it shows green. It was latent: Avalonia's compositor applies the UI
thread's batches in order, so an older operation reaching the render thread after a newer one was not seen in
practice. It matters now because an older operation's array may already be another frame's, lent again.

**The mechanism.** Each `UpdateFrame` makes an *offer*: the array, its geometry, its version, its `release`, and a
count of its readers. A draw operation made by `Render` is a reader of the newest offer until it is disposed; a draw
in progress is a reader of the offer it draws from. The render thread keeps `_copiedVersion`, the newest version it
has ever copied, which only moves forward, and `_cached`, the offer the cached image came from. A draw copies its own
offer only if that offer is newer than `_copiedVersion`; otherwise it draws the cached image, with the cached offer's
size and rows, whatever operation it is. An offer is then dead, and its array given back, when all three hold:

1. it is **superseded**, a later `UpdateFrame` having come;
2. it is **not the cached offer**, which is kept readable because the image is given back when the control leaves the
   window (§2.6) and must be made again on its return, and because a RetroArch preset started later asks the draw for
   the picture on show (`SlangRunner.Draw`'s `!_advanced`, §7.5);
3. it has **no readers, or its version is at most `_copiedVersion`**, in which case every reader it has will draw the
   cache and not it.

The test is made where each condition can change: in `UpdateFrame` for the offer just superseded, when an operation
is disposed, and after every copy for all the superseded offers still held, of which at most eight are kept; one
pushed out is never given back and is left to the collector, which costs an allocation, never a picture. The
bookkeeping has its own lock, held for a few comparisons and never across a copy or a draw, so `UpdateFrame` does not
wait for the render thread. `release` runs under that lock, on whichever thread made the offer dead, and must not call
back into the control. **The same array offered again**, as Moon and Mercury offer their PPU's buffer, is not given
back while a later offer, the cached one or a held one holds it
(`The_same_array_offered_again_is_not_given_back_while_it_is_current`).

**Mistress's side** is `EmuSen.Mistress/FrameHandOff.cs`, the hand-off of §4 over LunaP's `Latest<T>`. `Latest`
drops a stale frame by forgetting it, which is right for a frame nobody owns and wrong for a lent one, so each frame
carries a state, and whichever thread moves it out of *waiting* owns its array: the UI thread by presenting it, which
passes the array and `release` to the control, or the emulation thread by replacing it before the UI thread took it,
which gives it back at once. The `release` Mistress passes is `FrameHandOff.ReleaseFor(core)`: a *route* to the
session's core's `ReturnFrameBuffer`, taken once when the emulation loop starts, and `null` for a core that does not
lend. `EndSession`, called in `ShutDownCurrentSession` once the emulation thread has stopped, gives back a frame not
yet taken and then cuts the route. The picture still on screen after a game is closed or reset is let go only when a
later one is drawn over it, and by then it goes nowhere. The route exists for a reason found while writing this: a
`release` that named the core directly would have kept the ended session's core, native machine and threads with it,
reachable from the control for as long as its last picture stayed on screen — on the library screen, indefinitely
(`Once_a_session_ends_the_picture_left_on_screen_neither_keeps_its_core_nor_returns_to_it`).

**The evidence.** `GameFrameReleaseTests` holds operations and draws them in every order the rules distinguish,
recording what is given back: an unread offer when superseded, a drawn one when a newer is drawn, a held one when its
operation is disposed undrawn, and nothing twice. `FrameHandOffTests` wires MarsRT, the hand-off and the control as
`MainWindow` does, on the headless platform: in 400 frames on a seeded schedule, holding up to four operations, drawing
them late and out of order (83 of 187 draws late, 54 of them onto a changed picture) and disposing them at random, it
asserts after every frame that each array lent and not yet given back is byte-identical to the copy taken when it was
lent, that the core never lends one of them, and that every draw shows exactly the newest version drawn so far; the
lending reused 391 arrays, so a premature return had plenty to be overwritten by. Another counts 667 to 670 bytes
allocated a frame on that path against a 1,228,800-byte picture, and no array made after the first thirty frames.
Six mutants, each caught, are listed in `Mars_Native.md` §6.13.3.

**What this does not cover.** A frontend that passes `release` and goes on writing the array; a frontend that returns the
same lend twice across a re-lend (§16 of `EmuSen_Multicore.md` says why the lending cannot tell); an Avalonia that
rendered an operation after disposing it, which the operation checks and answers by drawing nothing. That check is made in the same hold of the lock that takes the draw's read: made in a hold of its own, as first written, a `Dispose` on another thread between the two could have given the array back just before the copy. That window was closed on reading the code, and no test reaches it. The live
compositor's order of rendering and disposing was not instrumented; the rules do not depend on it.

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

Timothy Lottes' CRT shader, in the public domain, ported from libretro's `crt/shaders/crt-lottes.slang` at its default parameters: Gaussian scanlines (`hardScan -8`) and pixels (`hardPix -3`), a small bloom (`0.15`), the barrel warp (`0.031`, `0.041`) with black outside it, and shadow mask 3 (the stretched VGA mask, dark `0.5`, light `1.5`), in linear light with the sRGB curve both ways. The port changed nothing but spelling: `vec` to `float`, the texture fetch to `source.eval` at a texel's centre, the output-pixel mask coordinate to SkSL's `coord`, and the parameters to constants. Offered for the NES, SNES and N64. *(2026-09-24: eleven of the constants are uniforms again, with the slang file's defaults and ranges, so a player can set them; §3.7.)*

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

### 3.7 Parameters a player may set on a built-in filter (2026-09-24)

A `ScreenFilter` may now list `Parameters`, each a `SlangParameter` (id, label, default, minimum, maximum, step), the same record a slang pass declares (§7.2), so a settings window draws one kind of slider for both. A parameter is an SkSL `uniform float` of the same name in whichever passes use it. `FilterChain` holds a value for each, the default until `SetParameters` gives another; a uniform a pass declares that is neither one of the chain's own (`inputSize` and the rest, §3.2) nor a parameter is still refused when the chain is built. `SetParameters(null)`, or leaving an id out, puts that parameter back to its default; an id the filter does not declare is ignored, so a value stored for another filter cannot break this one.

**Which filters have them, and why these.**

- **CRT (Lottes)** has the eleven that `crt-lottes.slang` declares as `#pragma parameter` and the port kept: `hardScan`, `hardPix`, `warpX`, `warpY`, `maskDark`, `maskLight`, `brightBoost`, `hardBloomPix`, `hardBloomScan`, `bloomAmount` and `shape`, with the slang file's defaults, ranges and steps (read from the pack's copy, lines 20 to 32) and labels written for a person rather than its ids. Its `scaleInLinearGamma` and `shadowMask` are not offered: the port fixed linear light and mask 3 (§3.4), and a slider for a mask the SkSL does not contain would change nothing.
- **The three monochrome LCDs** have the panel's response (`response`, 0 to 0.9) and the dots' shadow (`shadowOpacity`, 0 to 1); **the three colour LCDs** have the response. §3.5 records both as choices made by eye rather than measurements, so they are the numbers a player may reasonably disagree with. The palettes, matrices, luminance, dot coverage and stripe floor and gain are not offered: the first three are measurements, and the last three are the grid's geometry, which §3.5's brightness correction was tuned against.
- **Scanlines and Simple CRT** are single-pass `ShaderEffect`s (§3), not `ScreenFilter`s, and have none.

**Bit-identity at the defaults.** Turning constants into uniforms lets SkSL's compiler fold less, so the defaults could in principle draw differently. The existing render tests (`ScreenFilterRenderTests`: the DMG shades within 2, the GBC matrix within 3, Lottes' scanline contrast and black corners) pass unchanged; no frame was compared byte for byte against the constant build.

**Tests.** `A_lottes_parameter_set_while_it_runs_reaches_the_picture_without_a_new_frame`: a grey picture under Lottes, then `brightBoost` 0 set on the running `GameFrameControl` with no new frame, draws black at the next paint, and `null` draws the first picture again; the chain's own value is read back, and an undeclared id alongside is ignored. `A_dmg_whose_dot_shadow_is_set_to_zero_casts_none`: the shadow of §3.5's magnified case disappears, which shows a value reaching a filter's second pass. Mutants: a chain that ignores the values (both red), and a control that does not hand a new value to a running chain (the Lottes case red).

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
- `GameFrameReleaseTests.cs` — §2.8: draw operations held and drawn out of order, and the arrays given back.
- `GameFrameControlRenderTests.cs` — the real render path, added specifically to close the "nothing exercises this" gap that hid the §2.2 crash. Byte-identical repeated output, 120 consecutive frames without throwing, and the exact scanline pixel math.
- `BuiltInShadersTests.cs` — asserts the SkSL source itself, via the `InternalsVisibleTo` in `AssemblyInfo.cs`.
- `FramePresenterEffectCyclingTests.cs` — `NextEffect`'s cycle, without a window.
- `ShaderBenchTests.cs` — §8.1's bench runs a short case and reports every stage; skipped unless `EMUSEN_SHADER_BENCH=1` and a pack are given, since it needs a GL device.
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

- `Advance` uploads the core's picture into a ring of history images, with repeated rows expanded first (§2.7), because a slang pass sees the picture as the screen would. *(2026-09-24: retired for the frame control. The premise was wrong for the screen a CRT preset emulates, which receives a progressive field's rows once, and the doubled picture made some twenty presets emulate interlacing on every 240p N64 game. `SlangRunner` now advances the chain with the rows once; the chain keeps the argument. See §10.6.)*
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

**Why readback.** The chain ends by copying its last image into host memory, and the frontend is to draw that as an ordinary bitmap. Handing a Vulkan image to Avalonia's compositor directly would save the copy. However, it needs the compositor to be on Vulkan and to import the image, which Avalonia exposes only on some backends. The copy also keeps this runtime independent of what Avalonia is drawing with. Its cost is not yet measured inside Mistress. On the test device, ten frames of `crt-royale` (12 passes, 256×224 to 1024×896, readback included) took 24 ms, about 2.4 ms a frame. *(2026-09-24: measured stage by stage on a real GL context in §8, where the readback and what follows it are most of a light preset's frame, and the premise that the compositor must be on Vulkan is retired: a GL context imports the device's image, §8.5.1.)*

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

### 7.6 A preset's parameters, read without a device and set while it runs (2026-09-24)

**Reading them.** `SlangParameters.Read(preset)` loads every pass's source (§7.2) and compiles nothing, so a settings window can list a preset's parameters on a machine with no Vulkan device. `SlangParameters.Merge` is the rule both it and `SlangChain` use: in pass order, the first declaration of an id wins, and **a preset's own value for a parameter replaces its `Initial`**. So a preset that is only a `#reference` and four overrides (§7.1) reports those four values as its defaults, and a reset in the frontend returns to what the preset's author chose rather than to the bare shader's. `SlangChain.Parameters` used to list the pragmas' own initial values while the chain drew the preset's; its comment claimed the preset's. It now lists what it draws. `IsHeading` names a parameter whose range has no width (`[ LINE ]` 0 0 0 1), which many presets declare only to label a group of sliders in RetroArch's menu.

Measured on the pack of 2026-09-22 (§7), reading and not compiling, on the development desktop (`A_pack_preset_s_parameters_are_read_without_compiling`, with `EMUSEN_SLANG_PACK`): `crt-lottes` 13 parameters in under a millisecond; `crt-royale` 46 in 77 ms cold and 39 ms again; `crt-guest-advanced` 148, 2 of them headings, in 28 ms; and `bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV` **953**, 9 of them headings, in 61 ms cold and 34 ms again. `A_pack_preset_runs` now also requires `Read` to give exactly the chain's list for the four presets it builds, and it does. What 953 does to a window is the frontend's problem (`EmuSen_Settings_Reference.md` §4.48.2: 944 sliders built in 1.9 s, the building and not the read).

**Setting them.** `SlangChain.SetParameters(values)` sets each declared parameter to the value given for its id, or to its default when none is given; the next `Render` fills the blocks with them (§7.4's rule for a member named by a parameter is unchanged). `SlangRunner.SetParameters` may be called before the chain is built or from the UI thread under the frame control's lock; the runner hands the values to the chain at its next `Draw`, on the render thread, and **renders again even without a new frame**, so a slider moved while the game is paused shows its effect. `GameFrameControl.ShaderParameters` is the one property a frontend sets: it keeps a copy, gives it to the running filter chain (§3.7) and the running preset, and to whichever one is built next. Mistress sets it from `graphics.json` before it sets the filter or preset, whenever a game starts or its Shaders window changes a choice or a slider (`EmuSen_Settings_Reference.md` §4.48.3); a slider leaves the filter reference and the preset path as they were, so nothing is rebuilt and only the values move.

No value is clamped to its declared range here. The frontend's sliders cannot leave the range, and a preset may itself set a value outside a pass's declared range, which RetroArch draws as given.

**Tests.** `SlangChainTests.A_value_set_on_a_built_chain_reaches_the_next_render_and_leaving_it_out_returns_to_the_preset_s`: `Read` gives the preset's 0.2 as the default of a gain declared 1.0; the chain lists the same; it draws red 51, then 153 for 0.6, then 51 again for `null`, and ignores an undeclared id. `SlangFrameControlTests.A_parameter_set_on_the_control_reaches_the_drawn_preset_without_a_new_frame`: through the real headless render pass, 51, then 204 for 0.8 with no new frame, then 51. Mutants: the chain ignoring the values (two red); the runner not rendering again when only the values changed (the frame-control case red); `Merge` ignoring the preset's values (three red, including §7.4's override case).

---

## 8. What a shaded frame costs, and the levers ranked (2026-09-24)

A research pass: measure where a shaded frame's time goes, state what each candidate change should be worth before
measuring it, then measure it on a prototype or bound it. Nothing in the production path was changed except the
bench's listening points (§8.1). The prototypes live outside the repository and on the scratch branch
`shader-research-proto`; §8.5 says what each would take to build properly.

### 8.1 The bench, and what it measures

**The tool.** `EmuSen.WiseMan/Serenity/ShaderBench.cs` drives the real `GameFrameControl` draw path, the one Mistress's
render thread runs: `UpdateFrame`, `CaptureDrawOp`, and `DrawOp.RenderTo` onto a GPU surface of a real **OpenGL**
context, which is what Avalonia's Skia draws with under X11 (§2.5 was read under GLX). The context is made
headlessly, surfaceless, through EGL's device platform on a chosen card (`EMUSEN_BENCH_GL_DEVICE`, a substring of
`GL_RENDERER`; on the desktop `6800`, since the first device EGL lists is the processor's integrated one). A slang
preset runs on its own `SlangVulkan` device as in Mistress (§7.4, §7.5). A console host outside the repository,
`~/.cache/emusen/probe/shaders/shaderbench/`, compiles the same file and is named `EmuSen.WiseMan` so that Serenity's
`InternalsVisibleTo` admits it; `ShaderBenchTests` keeps the file compiling and running under the test harness
(gated, since it needs a GL device).

**The listening points.** `SlangProbe.Current`, null outside a bench, is told the CPU phases (the advance's start and
end, the render's start, descriptors bound, submission, fence waited, the readback copied, the Skia image made, the
flush) and is handed each command buffer at its start, after each pass and at its end, where the bench writes Vulkan
**timestamp queries**. Skia's own GPU work is timed with a `GL_TIME_ELAPSED` query around `RenderTo`. In production
each point is a static field read and a null test.

**The protocol.** Frames paced at 60 Hz, as Mistress's are (flat out, the RX 6800 clocks up and a Lottes pass takes
0.48 ms instead of the paced 0.69: the pacing is part of the measurement, not noise); 60 frames of warm-up, 300
measured; after each frame a `glFinish`, timed separately, so that no frame's GL work leaks into the next. The
picture is a gradient under a checker that moves every frame, so no stage can skip work. Each case is its own
process; cases are interleaved in rotated order over three rounds, every run under the bench lock
(`~/.cache/emusen/probe/mars-speed/bench.lock`); a table's figure is the median over rounds of each run's median.
Three extra frames after the measured ones are hashed off the surface, so that a prototype can be checked picture for
picture against production. Runner, case lists and raw results: `~/.cache/emusen/probe/shaders/` (`matrix.sh`,
`cases-*.txt`, `results-*.txt`, `summarise.py`).

**The stages** (all milliseconds on the render thread unless marked GPU):

| Stage | What it is |
|---|---|
| source copy | `GameFrameControl` copying the offered frame into an `SKImage` (§2.6) |
| advance | `SlangChain.Advance`: rows expanded (N64, §2.7), host copy into the staging buffer, record, submit, **wait** |
| chain wait | `SlangChain.Render`'s submission, from `vkQueueSubmit` returning to its fence signalled |
| passes (GPU) | first timestamp to the last pass's; readback (GPU): last pass to the end of `vkCmdCopyImageToBuffer` |
| host copy | `new byte[w·h·4]` and `MemoryCopy` out of the mapped readback buffer |
| image copy | `SlangRunner`: `SKImage.FromPixelCopy` of that array |
| draw image | `DrawImage` of the raster image onto the GPU canvas, which is where Skia uploads it |
| flush | `GRContext.Flush` (§2.5) |
| GL (GPU) | Skia's GL work for the frame: the upload's copy and the draw |
| total | the whole of `RenderTo`: what the render thread spends on the frame |

**What it does not measure.** Avalonia's compositor, the swap and vsync, and the window system; the GPU contention
between the slang device and the compositor under a live desktop (here only one frame's GL work is ever queued);
the emulation thread, and the cost of the collections in §8.3 to a heap the size of Mistress's (the bench's is
small); content: the synthetic picture exercises every stage but is not a game's.

### 8.2 The predictions, stated before any measurement

Written to `~/.cache/emusen/probe/shaders/PREDICTIONS.md` before the first run. The verdicts are §8.3 to §8.6's.

| | Prediction | Verdict |
|---|---|---|
| P1 | Light presets (Lottes, `lcd-grid-v2`): GPU passes under 10% of the frame; readback, copies and Skia's upload at least 60% | **Refuted** in its first half: Lottes' one pass is 27%, `lcd-grid-v2` 17%. Held in its second: 67% and 79% |
| P2 | `crt-royale` and Mega Bezel: GPU passes at least 50% | **Refuted** at 1080p on the RX 6800: 22% and 38% |
| P3 | 1080p totals: Lottes ~2, guest-advanced ~2.5, royale 3–4, Mega Bezel 6–10 ms; at 4K the transfers ~4× | Lottes 2.6, guest-advanced 2.7, **royale 2.5 (over-predicted)**, Mega Bezel 6.4; 4K readback 4.0×, but the frame **5.3×** (§8.3) |
| P4 | Each of the two submissions costs ≥0.1 ms beyond its GPU time | Held: the upload's record, submit and wait cost 0.36 ms around 0.04 ms of GPU work; the chain's fence wakes 0.09 ms after the GPU finishes |
| P5 | The per-frame readback array causes ≥1 gen2 collection a second | Held, by far: one every other frame, 30 a second |
| P6 | The SkSL built-ins cost the render thread under 0.5 ms, less than slang Lottes | Held: 0.09 ms against 2.59, for the same GPU work (0.75 ms GL, 0.69 ms Vulkan) |
| P7 | Handheld: GPU passes 3–5× the desktop's; Mega Bezel over 16.7 ms; royale 8–10 ms | **Refuted** on all three (§8.4): 2.4–9.8×, mostly above 5; Mega Bezel 13.3 ms; royale 6.5 |
| L1 | No readback (interop): −1.5 to −2.5 ms at 1080p, −5 to −8 at 4K | −1.4 at 1080p held; **4K −11.3 exceeded the range**, because the allocation it also removes costs more at 4K than predicted |
| L1b | Async readback hides the passes' time | Held: −1.2 (Lottes) to −3.0 ms (Mega Bezel), at a frame of latency |
| L1c | No array and no Skia copy: −0.5 to −1 ms at 1080p, and no gen2 | Held: −0.68, no collections |
| L2 | Not re-running for an unchanged frame: already so, gain ~0 | Held: a redraw costs 0.05 ms and runs no pass |
| L3 | Passes at source size: the preset decides; no general gain | Argued only; not measured |
| L4 | Shaderc is ≥70% of a build; a SPIR-V cache cuts ≥70%; a `VkPipelineCache` adds <10% on RADV | Held on all three (§8.6) |
| L5 | One submission instead of two: −0.05 to −0.15 ms | **Refuted**: −0.02 and −0.03 ms, inside the noise |
| L6 | Output at the window's size: already so | Held by reading `SlangRunner.Draw`; §8.3's 4K rows bound what a smaller output would save |
| L7 | Virtualising the Shaders window: 1.9 s to under 0.2 s | Not measured here (§8.8) |
| L8 | Built-ins moved to Vulkan: a regression while the readback stays; slang to SkSL impossible | Held by P6's figures and §3.2 |

### 8.3 The desktop: where a shaded frame goes

Ryzen 7 7700X, RX 6800 (RADV for the slang device, radeonsi for GL), Fedora 44, Mesa's shader cache warm. Paced at
60 Hz, three rounds, medians. The window is 1920×1080 (4K where marked), and the picture is letterboxed into it:

| Case | Out | Total | Source copy | Advance | Chain wait | Passes GPU | Readback GPU | Host copy | Image copy | Draw image | GL GPU | MB/frame | gen2 /300 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| SNES, none | 1234×1080 | **0.07** | | | | | | | | | 0.07 | 0 | 0 |
| SNES, built-in CRT (Lottes) | 1234×1080 | **0.09** | | | | | | | | | 0.75 | 0 | 0 |
| SNES, `crt-lottes` | 1234×1080 | **2.59** | 0.02 | 0.39 | 1.18 | 0.69 | 0.40 | 0.46 | 0.16 | 0.32 | 0.43 | 5.1 | 150 |
| SNES, `crt-guest-advanced` | 1234×1080 | **2.66** | 0.02 | 0.39 | 1.11 | 0.65 | 0.41 | 0.49 | 0.16 | 0.32 | 0.43 | 5.1 | 150 |
| SNES, `crt-royale` | 1234×1080 | **2.46** | 0.02 | 0.39 | 1.02 | 0.55 | 0.40 | 0.40 | 0.15 | 0.31 | 0.42 | 5.1 | 67 |
| SNES, Mega Bezel POTATO | 1234×1080 | **2.78** | 0.02 | 0.39 | 1.01 | 0.52 | 0.41 | 0.52 | 0.17 | 0.31 | 0.43 | 5.1 | 92 |
| SNES, Mega Bezel SMOOTH-ADV | 1234×1080 | **6.37** | 0.02 | 0.39 | 2.88 | 2.39 | 0.41 | 2.06 | 0.18 | 0.33 | 0.43 | 5.2 | 97 |
| GB, none | 1200×1080 | **0.06** | | | | | | | | | 0.06 | 0 | 0 |
| GB, built-in Game Boy LCD | 1200×1080 | **0.11** | | | | | | | | | 0.10 | 0 | 0 |
| GB, `handheld/lcd-grid-v2` | 1200×1080 | **2.09** | 0.01 | 0.38 | 0.83 | 0.36 | 0.40 | 0.46 | 0.15 | 0.28 | 0.42 | 5.0 | 150 |
| N64 1×, none | 1440×1080 | **0.11** | | | | | | | | | 0.10 | 0 | 0 |
| N64 1×, built-in CRT (Lottes) | 1440×1080 | **0.12** | | | | | | | | | 0.79 | 0 | 0 |
| N64 1×, `crt-lottes` | 1440×1080 | **3.46** | 0.04 | 0.94 | 1.25 | 0.73 | 0.47 | 0.39 | 0.21 | 0.34 | 0.49 | 7.1 | 300 |
| N64 1×, `crt-royale` | 1440×1080 | **3.22** | 0.04 | 0.80 | 1.01 | 0.49 | 0.47 | 0.50 | 0.20 | 0.34 | 0.49 | 7.1 | 99 |
| N64 4×, none | 1440×1080 | **1.03** | | | | | | | | | 0.76 | 0 | 0 |
| N64 4×, `crt-lottes` | 1440×1080 | **9.32** | 0.72 | 4.54 | 1.07 | 0.56 | 0.47 | 2.28 | 0.33 | 0.40 | 0.49 | 24.7 | 86 |
| N64 4×, `crt-royale` | 1440×1080 | **9.26** | 0.70 | 4.60 | 1.70 | 1.18 | 0.47 | 0.81 | 0.31 | 0.41 | 0.49 | 24.7 | 86 |
| SNES 4K, none | 2469×2160 | **0.08** | | | | | | | | | 0.16 | 0 | 0 |
| SNES 4K, built-in CRT (Lottes) | 2469×2160 | **0.09** | | | | | | | | | 1.87 | 0 | 0 |
| SNES 4K, `crt-lottes` | 2469×2160 | **13.76** | 0.02 | 0.20 | 3.49 | 1.83 | 1.61 | 7.57 | 1.29 | 1.22 | 1.66 | 20.4 | 150 |
| SNES 4K, `crt-royale` | 2469×2160 | **8.31** | 0.02 | 0.41 | 2.80 | 1.13 | 1.61 | 2.29 | 1.28 | 1.43 | 1.67 | 20.4 | 120 |
| SNES 4K, Mega Bezel SMOOTH-ADV | 2469×2160 | **12.93** | 0.02 | 0.19 | 7.04 | 5.34 | 1.62 | 2.36 | 1.24 | 1.33 | 1.67 | 20.5 | 150 |

(N64 frames are sent with their rows once and a repeat of two, §2.7: 640×240 at 1×, 2560×960 at 4×.)

**What the table says.**

- **A preset's frame is mostly not its shaders.** For every light-to-middling preset at 1080p the passes are
  0.36–0.73 ms of a 2.1–3.5 ms frame. The rest is fixed by the path, not the preset: the readback on the GPU
  (0.40 ms), two synchronous waits, a host copy into a new array, a second copy into Skia, and Skia's upload of it back
  to the same GPU (0.43 ms of GL time). The built-in filters, which have none of that, cost the render thread
  0.09–0.12 ms for the same order of GPU work.
- **The per-frame array is the most expensive single stage at large sizes.** `Render` allocates a new array the size
  of the picture every frame (5 MB at 1080p, 20 MB at 4K); each is a large-object allocation, and a gen2 collection
  follows every other frame. Its cost varies with the heap: 0.4–0.5 ms at 1080p for the light presets, **2.1 ms** for
  Mega Bezel (a larger heap: its 23 lookup images), and **7.6 ms** at 4K for Lottes, where the same copy into a
  reused array is 1.2 ms (§8.5). The collections pause every managed thread, the emulation thread included; the bench
  counts 0.1–0.2 ms of pause a frame on its small heap, and does not know what they cost Mistress's.
- **The N64 frame is dominated by its row expansion.** `Advance` expands the repeated rows on the processor into a new
  20 MB array at 4× and copies it into the staging buffer, and the upload of twice the rows takes 1.5 ms of GPU time:
  4.5 ms of a 9.3 ms frame. `GameFrameControl`'s own copy of the source, which the slang path never draws, is another
  0.7 ms.
- **Nothing on this desktop misses a 60 Hz frame**; the worst, Lottes at 4K, spends 13.8 ms of 16.7 on the render
  thread. The GPU is idle most of the frame even for Mega Bezel at 4K (7.0 ms of Vulkan and 1.7 of GL work).

### 8.4 The handheld

**The device and the run.** Legion Go S: Ryzen Z1 Extreme (RDNA 3 integrated graphics, one memory for both
processors), SteamOS on kernel 6.18.50-valve1, glibc 2.41, platform profile `custom`, energy preference
`balance_performance`, **on battery** (`power_supply` online 0), so its clocks are the battery's, not the charger's.
The benches are the self-contained linux-x64 publishes in `~/.cache/emusen/probe/shaders/deck-out/{base,proto}`
(natives asking for glibc 2.27 at most), run by `deck-run.sh 3` from `~/emusen-bench/shaders/` with the case lists
in `~/.cache/emusen/probe/shaders/deck/`: the same protocol as §8.1, three interleaved rounds under a lock of the
device's own, the picture letterboxed into the panel's 1920×1200. The run was started by the parent session. For the
record: a first attempt in the background died with the SSH session that started it, and the run was repeated in the
foreground; only the repeat's results exist, so this touches no number. Raw results, copied back:
`~/.cache/emusen/probe/shaders/handheld/`.

**The predictions for it**, written with the desktop's results in hand but before any run on the device:

- **H1.** The transfer stages cost more there, not less: one memory serves both processors, so the readback, the two
  host copies and Skia's upload compete with the GPU's own bandwidth; Lottes at 1371×1200 over 4 ms.
- **H2.** The passes are 3–5× the RX 6800's; Mega Bezel SMOOTH-ADV's alone over 8 ms, and with the path's fixed costs
  the frame over 16.7 ms.
- **H3.** The levers' order does not change, but interop gains more than on the desktop, since the GPU work it removes
  (readback and upload, 0.8 ms a frame here) is taken from a GPU that is then the bound.

**Where the frame goes** (the production build; stages as in §8.3; medians over three rounds of each run's median):

| Case | Out | Total | Advance | Chain wait | Passes GPU | Readback GPU | Host copy | Image copy | Draw image | GL GPU | gen2 /300 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| SNES, none | 1371×1200 | **0.16** | | | | | | | | 0.14 | 0 |
| SNES, built-in CRT (Lottes) | 1371×1200 | **0.30** | | | | | | | | 3.59 | 0 |
| SNES, `crt-lottes` | 1371×1200 | **9.39** | 0.42 | 5.29 | 4.46 | 0.61 | 1.70 | 0.74 | 1.00 | 0.60 | 150 |
| SNES, `crt-guest-advanced` | 1371×1200 | **9.43** | 0.43 | 4.94 | 4.07 | 0.61 | 1.71 | 0.71 | 1.01 | 0.62 | 150 |
| SNES, `crt-royale` | 1371×1200 | **6.53** | 0.38 | 3.70 | 3.11 | 0.29 | 0.89 | 0.40 | 0.64 | 0.30 | 75 |
| SNES, Mega Bezel POTATO | 1371×1200 | **9.50** | 0.43 | 4.27 | 3.43 | 0.57 | 1.98 | 0.76 | 1.07 | 0.60 | 98 |
| SNES, Mega Bezel SMOOTH-ADV | 1371×1200 | **13.26** | 0.32 | 9.73 | 9.21 | 0.17 | 0.79 | 0.37 | 0.54 | 0.16 | 44 |
| GB, none | 1333×1200 | **0.10** | | | | | | | | 0.14 | 0 |
| GB, built-in Game Boy LCD | 1333×1200 | **0.15** | | | | | | | | 0.20 | 0 |
| GB, `handheld/lcd-grid-v2` | 1333×1200 | **4.30** | 0.34 | 1.37 | 0.86 | 0.28 | 1.45 | 0.38 | 0.60 | 0.17 | 150 |
| N64 1×, none | 1600×1200 | **0.14** | | | | | | | | 0.17 | 0 |
| N64 1×, `crt-lottes` | 1600×1200 | **12.06** | 1.57 | 6.20 | 5.27 | 0.71 | 1.77 | 0.84 | 1.27 | 0.70 | 300 |
| N64 1×, `crt-royale` | 1600×1200 | **8.68** | 0.93 | 4.78 | 4.10 | 0.34 | 1.05 | 0.51 | 0.71 | 0.34 | 112 |
| N64 4×, none | 1600×1200 | **1.77** | | | | | | | | 0.39 | 0 |
| N64 4×, `crt-lottes` | 1600×1200 | **16.34** | 5.68 | 6.15 | 5.46 | 0.34 | 1.35 | 0.47 | 0.70 | 0.34 | 93 |

**What it says.**

- **On the handheld the passes are the larger half of a preset's frame**, the reverse of the desktop: Lottes' pass is
  4.46 of 9.39 ms, Mega Bezel's 9.21 of 13.26. The path's fixed costs are still 3–5 ms of the rest.
- **The GPU's clock moves between runs, and the transfer stages with it.** Battery power and a light load leave the
  integrated GPU in one of a few clock states, and the readback's GPU time falls into three clusters across runs
  (about 0.61, 0.29 and 0.17 ms for the same 6.6 MB), with Skia's GL time following it. Royale's rounds were 6.3,
  9.8 and 6.5 ms for this reason; a run is a sample of a clock state as much as of the code. The heavier the preset,
  the higher the clock: Mega Bezel's own readback is the fastest of any. The lever runs below were a separate run, in
  which the control's clocks sat lower in the transfer stages (Lottes' readback 0.17 ms, its total 7.12), so they are
  compared only within that run.
- **One case misses the frame.** N64 4× under `crt-lottes` spends a median 16.3 ms on the render thread, with the mean
  over 16.7 in two rounds of three (17.4, 17.7); the row expansion is 5.7 ms of it. Every other case fits, Mega Bezel
  with 3–4 ms to spare for the compositor, which the bench does not include.
- **The collections cost three times as much.** The per-frame arrays cause the same gen2 collections as on the
  desktop, but a light preset's pauses sum to about 100 ms over 300 frames against the desktop's 33.
- **The built-in filters are cheaper on the GPU as well**: the built-in Lottes is 3.59 ms of GL work to the slang
  pass's 4.46 of Vulkan work at the same size, and 0.30 ms of the render thread to its 9.39.

**The levers there** (the prototype build, one run of three rounds, medians; the control is the same binary with no
lever set, and every lever drew the control's pictures, the asynchronous ones one frame late, as on the desktop):

| Lever | Lottes | royale | Mega Bezel | N64 4× Lottes |
|---|---|---|---|---|
| none (the control) | 7.12 | 5.56 | 13.40 | 13.87 |
| `reuse` | 6.62 | | | |
| `direct` | 5.77 | | 12.09 | |
| `async` | 2.34 | | | |
| `direct,async,nosourcecopy` | 0.85 | 0.91 | 1.92 | |
| `interop` | 5.13 | | 11.20 | |
| `interop,async,nosourcecopy` | 0.22 | 0.34 | 1.06 | |
| `gpuexpand` | | | | 10.50 |
| `gpuexpand,direct,nosourcecopy` | | | | 8.22 |
| `gpuexpand,direct,async,nosourcecopy` | | | | 2.39 |
| `gpuexpand,interop,async,nosourcecopy` | | | | 1.17 |

`direct` gains −1.35 ms for Lottes (the desktop's −0.68) and −1.31 for Mega Bezel; `interop` −1.99 and −2.20. The
first lever of §8.8 takes N64 4× from 13.9 to 8.2 ms, and from the production run's 16.3, which missed frames, to well
inside the frame. The asynchronous rows gain the whole chain wait, 4.4–9.7 ms, which on this device is most of the
frame.

**The predictions' verdicts.**

- **P7**, passes 3–5× the desktop's: **refuted as a range.** The ratio runs from 2.4× (`lcd-grid-v2`) to 9.8× (N64 4×
  Lottes); per output pixel, the handheld's panel being 23% larger, 1.9× to 7.9×. Mega Bezel alone is inside (3.9×,
  3.1 per pixel); the light and middling presets are above it, 5.6–8.4×. Its second half, Mega Bezel over 16.7 ms:
  **refuted** (13.3 median, means 13.5–14.4). Its royale estimate (8–10 ms): **refuted**, 6.5 (the clock-state
  spread above reaches 9.8 in one round).
- **H1**, the transfer stages dearer and Lottes over 4 ms: **held.** The readback, the two host copies, Skia's draw
  and its GL work sum to 4.7 ms for Lottes against the desktop's 1.8; the frame is 9.4 ms.
- **H2**, the passes 3–5×, Mega Bezel's over 8 ms and its frame over 16.7: **partly held.** Mega Bezel's passes are
  9.2 ms (held); the range is P7's and refuted with it; the frame is 13.3 ms (refuted).
- **H3**, the order unchanged and interop gaining more because it frees a GPU that is the bound: **refuted in its
  reason, mixed in its figure.** On one memory the readback and the device-side copy that replaces it cost the same
  (0.17 against 0.16 ms), and the GL draw of the imported image costs what the upload-and-draw did (0.16 ms), so
  interop frees almost no GPU time here; its render-thread gain is larger than the desktop's for Lottes (−1.99 against
  −1.41) and smaller for Mega Bezel (−2.20 against −3.08). The order of the synchronous levers is unchanged; what
  changes is the weight of the asynchronous one (§8.8).

**A preset's build there** (`kind=load`, one filling round then three measured, medians):

| Preset | Build | Shaderc | Build, driver cache off | SPIR-V cache | Parallel passes | Both |
|---|---|---|---|---|---|---|
| `crt-royale` | 2,205 ms | 2,021 | 3,029 | 167 | 248 | 147 |
| Mega Bezel SMOOTH-ADV | 8,602 | 6,328 | 10,229 | 556 | 663 | 499 |

Builds take 1.8–1.9× the desktop's, Shaderc still most of it; either cache or parallel compilation brings Mega Bezel
from 8.6 s to about half a second, as on the desktop.

**What this does not cover.** The charger's clocks, and any control of the GPU's clock state (none was pinned, by
design, since a player's device is not pinned either); Game Mode's gamescope compositor, under which Mistress would
really run on this device; interop through Avalonia's own GL context; and the thermal state over a session
longer than the bench's minutes.

### 8.5 The levers, measured

Each lever was built as a prototype over a copy of Serenity (`~/.cache/emusen/probe/shaders/proto-src/`), chosen per
run by `EMUSEN_PROTO`, so a lever and its control are the same binary. Every lever below that does not add latency
drew **byte-identical pictures** to the control on the three hashed frames; the asynchronous ones drew the control's
pictures one frame later, as designed. Render-thread milliseconds, desktop, paced, three rounds, medians:

| Lever | Lottes 1080p | royale 1080p | Mega Bezel 1080p | Lottes 4K | N64 4× Lottes |
|---|---|---|---|---|---|
| none (the control) | 2.53 | 2.50 | 6.38 | 13.78 | 9.31 |
| `reuse`: one array, kept | 2.15 | | | 7.69 | |
| `direct`: Skia reads the mapped readback, no array, no Skia copy | 1.86 | 1.83 | 4.09 | 5.31 | |
| `onesubmit`: the upload recorded into the chain's submission | 2.52 | | | | 9.29 |
| `async`: two frames in flight, the one before shown | 1.33 | | | | |
| `direct,async` | 0.66 | 0.73 | 1.13 | 1.55 | |
| `interop`: GL imports the device's image, no readback | 1.12 | 1.10 | 3.30 | 2.48 | |
| `interop,async` | 0.27 | 0.39 | 0.78 | 0.32 | |
| `dontcare`: intermediate passes not cleared | 2.58 | 2.49 | 6.31 | | |
| `gpuexpand`: rows repeated by a blit on the device | | | | | 4.80 |
| `nosourcecopy`: no `SKImage` of the source while a preset draws | | | | | 8.99 |
| `gpuexpand,direct,nosourcecopy` | | | | | 3.02 |
| `gpuexpand,direct,async,nosourcecopy` | | | | | 1.18 |
| `gpuexpand,interop,async,nosourcecopy` | | | | | 0.64 |

GPU time a frame, from the timestamps and the GL query: the control's Lottes at 1080p is 1.09 ms of Vulkan and
0.43 of GL; with `interop` 0.65 and 0.02. At 4K, 3.45 and 1.66 against 1.99 and 0.05. The asynchronous rows move the
passes' time off the render thread, not off the GPU.

#### 8.5.1 No readback: the device's image imported by the GL context (`interop`)

**What was built.** The last pass's image is copied on the device (0.02–0.05 ms) into one of two images allocated
with `VK_KHR_external_memory_fd` as dedicated, exportable memory, released to the external queue family in layout
`GENERAL`; its file descriptor is imported into the GL context with `GL_EXT_memory_object_fd`
(`glImportMemoryFdEXT`, dedicated, optimal tiling, `glTexStorageMem2DEXT`) and wrapped for Skia with
`GRBackendTexture` and `SKImage.FromTexture`. Synchronisation in the prototype is the CPU waiting on the Vulkan fence
before GL draws, and the bench's `glFinish` before the image is written again two frames later.

**What it showed.** RADV and radeonsi agree on the image's layout: every picture is byte-identical to the readback's.
It removes the readback's GPU copy, the host copy, the Skia copy and Skia's upload: −1.41 ms at 1080p for Lottes,
−3.08 for Mega Bezel, −11.3 at 4K; and 0.85 ms of GPU work a frame at 1080p, 3.1 ms at 4K. With `async` the render
thread's whole cost is 0.27–0.78 ms.

**What building it would take.** Enabling the extension on the slang device; the two exported images and their GL
textures, made on the render thread in Avalonia's own GL context (the lease's `GRContext`, which is the context the
prototype's calls stand in for) or through Avalonia 12.1's `ICompositionGpuInterop.ImportImage` and
`CompositionDrawingSurface.UpdateWithSemaphoresAsync`, whose presence in 12.1.0 was checked in its assembly;
**GPU-side synchronisation** (`GL_EXT_semaphore_fd`: the device signals, GL waits before it draws and signals when it
has, the device waits before it writes that image again), since the prototype's CPU wait and `glFinish` are the
bench's, not a compositor's; and a fallback to the readback where the extensions are missing, which is Windows'
ANGLE (it would want an NT handle and D3D11) and macOS (IOSurface through MoltenVK). Mars_Gpu.md §16 decided the same
question for Mars's picture and did not build it, because there the copy was off the thread that bounds the frame
rate; here too the render thread is not the emulation thread, but the preset's whole cost is on it.

**What it does not show.** The prototype never ran under a compositor; whether Avalonia's GLX context accepts the
same import (the same radeonsi, so expected) is untested. Nor was a second vendor tried: NVIDIA and Intel's
proprietary and Mesa drivers were not in reach.

#### 8.5.2 No array, no Skia copy (`direct`), and the array kept (`reuse`)

`direct` makes the Skia image with `SKImage.FromPixels` over the mapped readback buffer, so nothing is copied on the
processor at all; Skia's upload reads the mapped memory (host-cached, as `SlangVulkan.CreateBuffer` prefers). It is
safe in the synchronous path because the buffer is written again only by the next `Render`, and `SlangRunner`
replaces the image in the same call before anything can draw it. −0.68 ms at 1080p, −2.28 for Mega Bezel, −8.47 at
4K, and no collections. `reuse` alone (one array kept, still copied twice) gets about half: −0.38 at 1080p, −6.09 at
4K, which measures the allocation by itself: at 4K the new array, not the copy, is 6 ms of the 7.6.

#### 8.5.3 Two frames in flight (`async`)

Two command buffers, fences, readback buffers and present images; a frame waits for the previous one's fence before
it writes the staging buffer, submits its own and returns, and the picture shown is the previous frame's. Gains the
whole chain wait: −1.21 ms for Lottes alone, −2.96 on top of `direct` for Mega Bezel. **The cost is a frame of
latency**, 16.7 ms more between a button and the picture, which in an emulator is a real cost and why this is ranked
below levers that have none. RetroArch pays the same kind of cost by default with three swapchain images (§8.7).

#### 8.5.4 The rows repeated on the device (`gpuexpand`), and a negative result on the way

The first prototype recorded one buffer-to-image copy region per repeated row, 1,920 regions at 4×: −2.6 ms, with the
upload's GPU time still 1.0 ms and 2.7 ms of `Advance` spent outside the wait, which holds a 9.8 MB copy and the
recording of the regions. That RADV makes each region a meta operation of its own is the likely reason; it was argued
from the numbers, not confirmed in the driver. **Kept as a negative result.** The second uploads the rows once into an image of their own height
and blits it to the full height with nearest filtering, which for an integer factor duplicates each row exactly:
−4.5 ms (9.31 to 4.80), the upload's GPU time halved (1.48 to 0.79 ms), identical pictures. `nosourcecopy` removes
`GameFrameControl`'s `SKImage` of a source the preset never draws: −0.3 ms at N64 4× (the stage itself is 0.7 ms; the
rest moved into noise), and near nothing for a SNES frame.

#### 8.5.5 Levers that bought nothing

- **One submission (`onesubmit`).** The upload's record, submit and wait (0.23–0.39 ms) disappear from `Advance` and
  reappear in the chain's wait; the totals move by 0.02 ms. Refuted (L5).
- **Not clearing intermediate passes (`dontcare`)**, as RetroArch does (§8.7): the passes' GPU time is unchanged
  within noise on RDNA 2 (Lottes 0.66 against 0.63, Mega Bezel 2.36 against 2.38), whose clears are fast clears.
  Recorded so that it is not tried again for speed; it is still what the spec allows.
- **Descriptor updates and uniform fills every frame** were not prototyped: `render.size.bind` is 0.002 ms for one
  pass and 0.25 ms for Mega Bezel's 48 (0.30 at 4K), which bounds what caching them could save. RetroArch rewrites
  them every frame as well.
- **Not re-running the chain for an unchanged frame** is already done (`SlangRunner.Draw` renders only for a new
  frame, a new size or new parameter values): a redraw with no new frame costs 0.05 ms and runs no pass
  (`EMUSEN_BENCH_EVERY=2`). The built-in `FilterChain`, by contrast, redraws its passes on every repaint (0.69 ms of GL
  for Lottes at 1080p), which the render thread does not see but the GPU does.

### 8.6 A preset's build time

`kind=load`: after a small shader has warmed Shaderc and the device, `new SlangChain` is timed whole, then its parts
separately. Three rounds, medians, after a round that filled the caches:

| Preset | Passes | Build | Shaderc | Build, driver cache off | SPIR-V cache | Parallel passes | Both |
|---|---|---|---|---|---|---|---|
| `crt-lottes` | 1 | 105 ms | 79 | 116 | 24 | 102 | 30 |
| `handheld/lcd-grid-v2` | 1 | 98 | 76 | 105 | | | 27 |
| `crt-guest-advanced` | 12 | 970 | 926 | 1,052 | 53 | 99 | 56 |
| `crt-royale` | 12 | 1,238 | 1,089 | 1,679 | 141 | 150 | 112 |
| Mega Bezel SMOOTH-ADV | 48 | 4,641 | 3,570 | 6,141 | 559 | 579 | 478 |

- **Shaderc is 75–95% of a build** (L4 held). A **SPIR-V cache**, keyed by a hash of each stage's expanded text, takes
  a repeat build of Mega Bezel from 4.6 s to 0.56, royale from 1.24 s to 0.14 (L4 held).
- **Compiling the passes in parallel**, which the prediction did not consider, does almost as much and helps the
  *first* build too: Mega Bezel 0.58 s, royale 0.15, on this 16-thread processor. Shaderc ran from many threads at
  once (`Parallel.For` over the passes) without error; the pictures of a preset so built were not compared.
- **A `VkPipelineCache` buys nothing here.** With Mesa's disk cache turned off (`MESA_SHADER_CACHE_DISABLE=true`)
  the driver's compilation is 0.44 s of royale's build and 1.5 s of Mega Bezel's, and a persisted pipeline cache gives
  back exactly the warm figure (1,230 against 1,238 ms; 4,639 against 4,641); with Mesa's cache on, which is the
  default, there is nothing left for it. It would matter on a driver without a disk cache, which none here is.
- A build runs off the render thread (§7.5), so its time is a wait for the filtered picture, not a stall; but it is
  what a player browsing presets in the Shaders window waits for at each one.

### 8.7 How RetroArch does it

Read from RetroArch's source at commit `ce5544fd` (`~/Projects/retroarch-reference`; no code taken):

- **It presents the chain directly.** The last pass's pipeline is built against the swapchain's render pass
  (`gfx/drivers_shader/shader_vulkan.cpp:2886-2888`) and drawn inside the frame's render pass on the backbuffer
  (`gfx/drivers/vulkan.c:7024-7028`); host memory is touched only for screenshots and recording. The glcore driver
  likewise draws its last pass into the default framebuffer. That is §8.5.1's lever, which RetroArch has because it
  owns the swapchain; EmuSen does not, since Avalonia does.
- **The CPU never waits for the frame it just submitted.** Frames in flight equal the swapchain images, 3 by default
  (`config.def.h:390-392`), each with its own command buffer and fence, and the frame waits only on the fence of the
  frame that used its slot before (`gfx/common/vulkan_common.c:1527-1542`). That is §8.5.3's lever, with its latency.
- **One command buffer carries the whole frame**: upload, passes, menu and present transition, one submission. §8.5.5
  found no gain in merging EmuSen's two.
- **The core's frame is written once**, into a host-visible linear texture per frame in flight that the first pass
  samples directly, or into it by the core itself through `get_current_software_framebuffer`, with a staging copy only
  where linear sampling is not supported.
- **It re-runs the chain for a duplicate frame**, and when paused it pushes the cached frame through the whole chain
  again (`gfx/video_driver.c:3335-3353`); EmuSen already does better here (§8.5.5).
- **SPIR-V is cached on disk**, keyed by a hash of the vertex and fragment source, under the cache directory when one
  is configured (`gfx/drivers_shader/slang_process.cpp:770-807`, `slang_cache.cpp:27-40`). The **pipeline cache is
  created empty and never saved** (`gfx/drivers/vulkan.c:4719-4727`; `vkGetPipelineCacheData` appears nowhere in its
  drivers), which agrees with §8.6: on drivers with their own disk caches a persisted one adds nothing.
- **Intermediate passes are not cleared** (`LOAD_OP_DONT_CARE`, `shader_vulkan.cpp:93-99`), which §8.5.5 found
  worth nothing on RDNA 2. A pass without a scale type is source ×1, the last viewport ×1
  (`shader_vulkan.cpp:4150-4160`), as §7.1 reads them.

Every cited line above was re-read at that line. The uncited statements (the glcore driver, the one command buffer,
the linear upload texture) are from a subagent's reading of the drivers and were not re-derived line by line.

### 8.8 The ranking, and the order of work

Gains are the desktop's, render thread, 1080p SNES unless said; effort is judged from the prototypes.

| Rank | Lever | Measured gain | Latency | Effort | Risk |
|---|---|---|---|---|---|
| 1 | **Skia reads the mapped readback (`direct`), the rows repeated by a blit (`gpuexpand`), no source image under a preset (`nosourcecopy`)** | −0.7 ms (Lottes), −2.3 (Mega Bezel), −8.5 at 4K; N64 4× 9.3 → 3.0 ms; no gen2 collections | none | small: ~60 lines in `SlangChain`, `SlangRunner`, `SlangVulkan`, `GameFrameControl` | the mapped buffer's lifetime (argued in §8.5.2; wants a test that a redraw never shows the next frame) |
| 2 | **A SPIR-V cache and passes compiled in parallel** | Mega Bezel's build 4.6 s → 0.48; royale 1.24 → 0.11 | — | small: a cache directory under `DataStore`, a content hash, `Parallel.For` | a stale cache is impossible by construction (content-keyed); disk use a few MB |
| 3 | **Interop, synchronous** | −1.4 ms (Lottes), −3.1 (Mega Bezel), −11.3 at 4K; −0.85 ms of GPU work (−3.1 at 4K) | none | large: extension, import in Avalonia's context, semaphores, per-platform fallback | only a real window can prove it under the compositor; one vendor tried |
| 4 | **Two frames in flight** | on top of 1: −1.2 to −3.0 ms; on top of 3: to 0.27–0.78 ms total | **+1 frame** | moderate: slots, fences, per-slot buffers, resizes | latency is a player-visible cost; worth it only where the render thread is the bound |
| 5 | Virtualising the Shaders window's sliders (L7) | not measured here: settings reference §4.48.2's 944 rows in 1.9 s, ~2 ms a row, against the ~25 rows a window shows | — | moderate (a LunaP list of rows and headings) | UI only |
| — | One submission; not clearing; caching descriptors; a pipeline cache; re-running only new frames | ≈0, or already done | | | recorded so as not to be re-proposed |

*The first two were built into production on 2026-09-24, with the cache in SQLite rather than files: §9.*

**The recommended order** is the table's. The first two are small, exact (identical pictures), cost no latency and
between them remove most of a light preset's avoidable frame cost and nearly all of a heavy preset's build time. The
third is the one that makes a slang preset cost what a built-in filter costs. ~~It is the one the handheld is most
likely to need (H3); it should be decided on §8.4's numbers once they exist.~~ *Retired 2026-09-24 by §8.4's
numbers:* on the handheld it frees almost no GPU time and gains −2.0 to −2.2 ms of the render thread, about what the
first lever gains there (−1.3) plus the upload, so it does not move up; it stays third, for the desktop's large windows.
The fourth trades latency for render-thread time and should come last, if at all, and then as a setting.

**The handheld's weights** (§8.4), gains against its own control: the first lever −1.35 ms for Lottes and −5.65 for
N64 4× Lottes, which takes the one case that missed frames (16.3 ms) well inside the frame (8.2); the second, Mega
Bezel's build 8.6 s → 0.5; the third −2.0 to −2.2. The fourth is worth far more there than on the desktop, because
the passes' wait is most of the frame (4.4–9.7 ms): with the first it leaves 0.85–1.92 ms. Whether a frame of latency
is worth that on the handheld is a player's choice, which is the argument for making it a setting rather than a
default; no case measured there needs it to stay inside the frame once the first lever is in.

### 8.9 What this does not cover

- **The handheld** (§8.4) was measured on battery, headlessly; not on the charger, not under Game
  Mode's compositor, and with its GPU's clock state left to the driver, which moves the transfer stages by up to 3×
  between runs.
- **Mistress itself**: no number here was read in a real window. The bench stands in for the render thread; the
  compositor's own work, the swap and vsync, and the UI thread's contention for the control's lock are outside it.
- **The emulation thread**: the gen2 collections of §8.3 pause it, and how long they pause it on Mistress's heap was
  not measured.
- **Other GPUs and drivers**: one discrete AMD card, RADV and radeonsi. The integrated card of this desktop was not
  used, by standing instruction; NVIDIA, Intel, Windows and macOS were not in reach.
- **Game content**: a synthetic moving picture, not captured frames; a shader's GPU time can depend on the picture,
  though the path's fixed costs cannot.
- **Correctness of the presets themselves** against RetroArch's output, still §7.4's open question; here the
  prototypes were compared only against production, picture for picture.

## 9. The first two levers, built (2026-09-24)

§8.8's first two levers, built into the production path on the branch `shader-levers`: the approach of the
prototypes (§8.5.2, §8.5.4, §8.6), rebuilt, not their code. The prototypes chose a lever by environment variable
over a copy of Serenity; here each is simply what the path does. Lever 3 (interop) and lever 4 (two frames in flight)
were not built (§9.8).

### 9.1 The readback lent to Skia

**What it does.** `SlangChain.RenderImage` runs the passes and returns an `SKImage` made with `SKImage.FromPixels`
over the mapped readback buffer itself: no array, no host copy, no Skia copy. `Render`, which returns an array, stays
for the tests and copies out of a readback of its own (the *scratch* one, never lent).

**The lifetime rule.** The mapped memory is written again by a later render, so an image must never read it after
that. Each readback is a `ReadbackSlot` that is *held* from the moment an image is made over it until Skia's raster
release call (the delegate `FromPixels` takes), on whatever thread Skia makes that call. A held slot is never
written: a render takes a slot that is not held and is large enough, makes a new one if none is, up to three
(`MaxLent`), and past that renders into the scratch readback and returns a copy. `CopiedImages` counts those copies;
a test asserts that steady drawing makes none, and the bench does not report the count. A slot grown for a larger view is retired, not resized,
if an image holds it; a chain disposed under a held slot leaves the slot to be freed by the release. `SlangRunner`
lets its last image go *before* rendering the next, so in steady state one readback serves every frame.

**What Skia does with the memory, measured rather than assumed.** On a direct GL context (the bench's, as §8.1's) a
raster image drawn onto a GPU surface is uploaded **at the draw**, not at the flush, and its pixels are released at
the image's `Dispose`, before any flush: an image drawn, its memory then rewritten and the context flushed, shows the
first contents; the release call comes at `Dispose` with no flush between. So under `GameFrameControl`, which draws
and flushes within one `RenderTo` (§2.5), nothing reads the mapped memory once the draw returns, and one slot would be
enough. The held flag does not rest on that: it makes the rule hold for a Skia that defers the upload to a flush (a
recording context, another backend), where a second slot would then be taken.

**A barrier production lacked.** The readback's copy is now followed by a buffer barrier to `HOST_READ` at the host
stage. A fence's signal does not by itself make a transfer's writes visible to the host; the old path read the
buffer without it. On RADV's host-coherent memory its absence changes nothing observable (its mutant survived,
§9.7), and the validation layer does not track host access; it is there because the memory model asks for it.

### 9.2 The rows repeated on the device

*(2026-09-24: the frame control no longer asks for this; since §10.6 the runner gives the chain a progressive
field's rows once. What follows still describes `SlangChain.Advance` with a repeat above one, which the tests use.)*

`Advance` uploads a frame's rows once, into an image of their own height (`_rows`), and blits that image to the
history image's full height with nearest filtering; the staging buffer is the frame's size, not twice it. For an
integer repeat *r* the blit is exact: destination row *y* samples source coordinate (*y* + ½)/*r*, which is never a
whole number, so the nearest texel is ⌊*y*/*r*⌋ with no rounding case for a driver to decide. A frame with mipmapped
input gets its chain generated after the blit, as before. The upload is reshaped when the width, the rows **or the
repeat** change, not only the height, since the same height can be 480 rows once or 240 twice
(`A_change_of_repeat_at_the_same_height_is_uploaded_as_the_new_shape`).

### 9.3 No image of the source under a preset

`GameFrameControl` made an `SKImage` of every new frame before offering it to the preset, which never draws it
(§8.3: 0.7 ms at N64 4×). Now, when a preset is built, drawable (`SlangRunner.Ready`) and no built-in filter chain is
set, no image is made, and the one made earlier (while the preset was still building) is dropped at the first new
frame, since it is then older than the frame shown. If the preset later declines to draw (it failed mid-draw, or was
turned off), the image is made then, from the offer §2.8 keeps readable for exactly that; turning a preset off with
no new frame therefore shows the newest frame, not the one copied before the preset was built
(`A_preset_that_draws_takes_no_source_image_and_turning_it_off_shows_the_newest_frame`). With a built-in filter set as
well, nothing changed: its chain keeps the frames it looks back at.

### 9.4 The SPIR-V cache, in SQLite

**Why SQLite.** A cache of compiled shaders is program-written data, and `EmuSen_Stack.md` §1 gives that to SQLite.
The prototype's loose files are not kept.

**Where the driver lives.** §2.3's rule is that a leaf may hold the contract and not the driver. `EmuSen.Serenity`
takes `Microsoft.Data.Sqlite` and `SQLitePCLRaw.bundle_e_sqlite3` itself, at `EmuSen`'s versions, because it is not a
leaf in that sense: it already references seven packages (three with natives) and three projects. It closes no cycle,
since a package is not a project. And no program gains a library: `EmuSen` references Serenity and both packages, and
every program that references Serenity (Mistress, Hotaru, WiseMan) references `EmuSen`. The alternative, an interface
here and the SQLite class in `EmuSen` handed in by each frontend and by the bench, buys a separability nothing uses.

**Where the database lives.** `home/Shaders/spirv-cache.db` (`SpirvCache.DefaultPath`, `DataStore.Shaders`), beside
the packs it serves and outside the pack's own folder, which a pack update replaces whole (§4.41 of the settings
reference); like `games.db`, under Galaxia's data home, not `/tmp`. `SlangRunner` opens one cache for the process on
the first build.

**The key, and why no row can be stale.** A row is keyed by the SHA-256 of, each part length-prefixed: a format tag;
the compiler's *identity* (the loaded Shaderc library's own SHA-256, found among the process's modules; the SPIR-V
version it reports; the binding's version; and `SlangCompiler.Options`, the string of every option a compile is
set); the stage; the file name the compiler is given; and the stage's text after its includes. The options string is
built from the same constants `Compile` uses, so neither can change without the other. **EmuSen passes Shaderc no
defines**; the string says `defines=none`, so that adding any means changing it, and with it every key. A changed
shader, an updated pack, a new Shaderc or a new option is a new key, never a changed row; rows are only ever inserted
(`INSERT OR IGNORE`) or deleted.

**A damaged row or file.** Each row carries the SHA-256 of its SPIR-V; a row whose bytes do not match, or that does not
begin with SPIR-V's magic, is deleted and compiled again, never handed to the device. A file that is not a database,
or that SQLite reports corrupt, is moved aside to `spirv-cache.db.damaged` and a new one begun; if even that fails,
the cache is off for the process and every stage is compiled. **Nothing the database does can stop a build**: every
failure falls back to Shaderc.

**Locks and concurrency.** One connection per `SpirvCache`, its statements under a lock; the compiles themselves run
outside it. The journal is WAL, so another process (Hotaru beside Mistress) reads while one writes, and a key is the
primary key, so two builders of the same stage cannot both insert it. A statement blocked by another process waits at
most about a second (`busy_timeout` 250 ms under a one-second command timeout) and is then skipped; the third blocked
statement in the process turns the cache off, so a database held for good costs a build about three and a half seconds
(measured, §9.7), not one per stage.

**The bound.** 64 MB of SPIR-V (`SpirvCache.DefaultLimit`). Past it, the least recently used rows go until three
quarters of the bound remains; a hit refreshes a row's use at most hourly, so a warm build writes almost nothing.
The five presets §8.6 builds (Lottes, `lcd-grid-v2`, guest-advanced, royale, Mega Bezel SMOOTH-ADV) fill 5.6 MB in
140 rows, so the bound holds about sixty presets of that weight. A pack update's new keys push the old version's rows
out; they do not grow the file without end.

**The schema** is `EmuSen.Serenity/Slang/spirv-cache-schema.sql`, embedded in the assembly rather than copied beside
it, so the cache has no file of its own to be missing. `PRAGMA user_version` is 1; a file of another version is
emptied and begun again, which a cache may do (§4.3 of `EmuSen_Stack.md` is the gap this does not have).

### 9.5 Passes built in parallel

`SlangChain` builds its passes with `Parallel.For` on every processor but one (`DefaultBuilders`), leaving one for the
emulation thread. What a pass's build touches is its own: its source file, a Shaderc compiler made for the call, and
Vulkan objects whose creation needs no external synchronisation on the device. A pass is recorded in the chain before
its objects are made, so a failure part way frees what was made; **on any failure every pass built is freed** (the
serial build leaked them before) and the error thrown is the lowest-numbered broken pass's, the one a serial build
would have met first (`A_parallel_build_that_fails_says_what_a_serial_build_would`). §8.6 noted that the pictures of a
preset built this way had not been compared; they now are, for royale, guest-advanced and Mega Bezel SMOOTH-ADV
(`A_preset_built_in_parallel_draws_what_one_built_pass_by_pass_draws`).

### 9.6 What it measured

**The frame.** These are §8.3's 22 cases, interleaved: for each case the production build (`57937b2`) and the lever's
(`d564dd8`) ran back to back, in an order that rotated. There were three rounds under the bench lock. Each figure is
the median over rounds of each run's median, render thread, in milliseconds. The raw results and scripts are in
`~/.cache/emusen/probe/shaders/levers/` (`results-lever1.txt`, `matrix-ab.sh`, `ab.py`). The prototype's column is
§8.5's, measured against its own control:

| Case | Production | Built | Change | Prototype's change | gen2 /300, before → after | Pictures |
|---|---|---|---|---|---|---|
| SNES `crt-lottes` 1080p | 2.53 | **1.82** | −0.71 | −0.67 (`direct`) | 150 → 0 | same |
| SNES `crt-guest-advanced` | 2.69 | **1.90** | −0.79 | | 150 → 0 | same |
| SNES `crt-royale` | 2.58 | **1.77** | −0.82 | −0.67 | 67 → 0 | same |
| SNES Mega Bezel POTATO | 2.72 | **1.85** | −0.86 | | 91 → 0 | same |
| SNES Mega Bezel SMOOTH-ADV | 6.25 | **3.94** | −2.31 | −2.29 | 98 → 0 | same |
| GB `lcd-grid-v2` | 1.97 | **1.34** | −0.63 | | 150 → 0 | same |
| N64 1× `crt-lottes` | 3.45 | **1.99** | −1.46 | | 300 → 0 | same |
| N64 1× `crt-royale` | 3.12 | **1.97** | −1.15 | | 99 → 0 | same |
| N64 4× `crt-lottes` | 9.28 | **3.07** | −6.20 | −6.29 (9.31 → 3.02) | 86 → 0 | same |
| N64 4× `crt-royale` | 9.43 | **3.69** | −5.73 | | 86 → 1 | same |
| SNES 4K `crt-lottes` | 13.56 | **5.28** | −8.29 | −8.47 | 150 → 0 | same |
| SNES 4K `crt-royale` | 8.27 | **4.45** | −3.82 | | 120 → 0 | same |
| SNES 4K Mega Bezel SMOOTH-ADV | 13.05 | **9.23** | −3.82 | | 150 → 0 | same |

The nine cases with no preset (no filter and the built-in filters, for SNES, GB, N64 at both multiples and 4K) moved
by at most 0.02 ms and drew the same pictures.

The stages moved where they should:

- The host copy (0.41–7.52 ms) and the image copy (0.15–1.45 ms) are gone.
- The source copy under a preset is now 0.00 ms; it was up to 0.71.
- `Advance` fell from 4.5–4.7 to 1.45–1.50 ms at N64 4×, and from 0.75–0.88 to 0.24 at 1×.
- Skia's `DrawImage` grew by 0.04–0.06 ms at 1080p, since its upload now reads the mapped buffer rather than its own
  copy. At 4K it moved by −0.19 to +0.07. Its GL time is unchanged.

Most of what is left of a light preset's 1.8 ms is the readback, the two waits and Skia's upload, which only interop
(§8.5.1) removes. Allocation a frame fell from 5–25 MB to 3.6 KB for a one-pass preset and to 15–118 KB for a
multi-pass one. The rest is `Bind`'s per-pass lists, which this lever did not touch. The one gen2 collection left
came in one of N64 4× royale's three runs.

The built lever matches the prototype within 0.2 ms wherever the two overlap.

**Against the predictions.** These were written to `~/.cache/emusen/probe/shaders/levers/PREDICTIONS.md` before the
first run:

- The 1080p range (−0.55 to −0.80) held for Lottes and guest-advanced. Royale and POTATO gained more than it
  (−0.82, −0.86).
- Mega Bezel's range held.
- The 4K royale and Mega Bezel range (−1.8 to −3.5) is **refuted**: both gained −3.82. At 4K their host copy and
  image copy (2.3 and 1.3–1.4 ms) cost more than their arrays' allocation, not less.
- N64 4× Lottes held; royale missed its range by 0.09 ms. N64 1× Lottes gained more than predicted (−1.46 against at
  most −1.3).
- The prediction of no collections and under 10 KB a frame is **refuted in both halves**: by the one collection, and
  by the multi-pass presets' lists.

**For the handheld.** N64 4× under Lottes missed the frame there (§8.4, 16.3 ms), and the prototype of this lever took
it to 8.2 ms against a 13.9 ms control. On the desktop the built lever equals the prototype. It has not been run on
the handheld (§9.8).

**The build.** This is `kind=load` over the same five presets as §8.6, interleaved, three rounds, medians in
milliseconds. One round filled a warm database first; *cold* means a new, empty database for each run. The results
are in `results-load2.txt` and `loadsum.py`:

| Preset | Production | Built, serial, no cache | Parallel | Parallel, cold | Cache, serial | **Both** | Prototype's both (§8.6) |
|---|---|---|---|---|---|---|---|
| `crt-lottes` | 99 | 99 | 103 | 105 | 23 | **23** | 30 |
| `handheld/lcd-grid-v2` | 103 | 99 | 99 | 101 | 23 | **23** | 27 |
| `crt-guest-advanced` | 980 | 970 | 100 | 146 | 54 | **55** | 56 |
| `crt-royale` | 1,230 | 1,242 | 156 | 175 | 137 | **109** | 112 |
| Mega Bezel SMOOTH-ADV | 4,663 | 4,670 | 582 | 637 | 548 | **475** | 478 |

**A player's first build of a preset** is the *parallel, cold* column: Mega Bezel from 4.66 s to 0.64, royale from
1.23 s to 0.18. **Every later build** is the *both* column: 0.48 s and 0.11.

Storing in SQLite costs a cold build 2–55 ms over parallel compilation alone: creating the database, and up to Mega
Bezel's 90 inserts. On a warm build it costs nothing measurable. The warm builds match the prototype's loose files within
3 ms for the multi-pass presets. The one-pass presets are 4–7 ms faster than the files. The likely reason is the
prototype's per-file existence check and open against one indexed lookup; that was argued, not isolated.

**Against the predictions** (the same file, written before the lever was first built):

- The cold and warm ranges held for every multi-pass preset.
- The one-pass presets were faster than their warm range (23 against 25–35 ms). That put them outside the ±10%
  predicted against the file cache.
- The database's size (5–7 MB) held, at 5.6.

Built serially and uncached, the built chain matches production within 1%. So the parallel column measures the lever,
not a change in what a pass's build does.

### 9.7 The evidence

**Pictures.** Every case of §8.3's matrix drew byte-identical pictures before and after, on all three hashed frames of
all three rounds: 22 of 22. `ab.py` exits non-zero on any difference. The tree with both levers ran the same 22 cases
once more against production (`results-final-pictures.txt`), with the same result.

In the suite (`SlangReadbackTests`, `SpirvCacheTests`), these are compared:

- the image over the readback, against the copy `Render` gives;
- rows repeated on the device, against rows repeated on the host, for repeats 2, 3 and 4, sampled nearest and
  through a mip chain;
- `crt-lottes` and `crt-royale` over a 2560×960 N64 frame repeated twice, and `crt-lottes` over 640×240, drawn at
  1440×1080 (`A_pack_preset_over_an_n64_frame_draws_the_same_from_rows_repeated_on_the_device`). N64 4× under a pack
  preset is the case the handheld misses frames on;
- a change of repeat at the same height;
- a cached build against an uncached one;
- an edited include against a fresh build;
- a parallel build of royale, guest-advanced and Mega Bezel against a serial one, parameters and three frames each.

**Lifetimes.**

- `An_image_still_held_keeps_its_picture_while_later_frames_are_rendered` holds four images, one past the limit. Each
  is rendered into its own readback, or into the copy. The test checks every held image's pixels after all four
  renders, and again after one image is let go and a fifth is rendered.
- `An_image_outlives_its_chain_with_its_picture_intact` disposes the chain under a held image.
- `A_running_preset_draws_every_frame_from_one_readback_and_copies_none` runs a `SlangRunner` for thirty frames and
  thirty redraws.
- `On_a_gl_canvas_every_draw_shows_its_own_frame_even_when_skia_uploads_it_again` draws on a real GL context (the
  bench's EGL one; gated on `EMUSEN_BENCH_GL_DEVICE`). After each frame it purges Skia's resources and redraws with no
  new frame, so that Skia uploads from the mapped memory a second time. Every draw shows its own frame.

Before any of this was built, a scratch test on the same context established what Skia does (§9.1): it uploads at the
draw, it releases at `Dispose`, and a rewrite before the flush is not shown.

**The validation layer.** It ran as `VK_LAYER_KHRONOS_validation` with `VK_LAYER_VALIDATE_SYNC=1`,
`VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1` and the duplicate-message limit off. It covered six short bench runs
through the real draw path: Lottes, royale, Mega Bezel, `lcd-grid-v2`, N64 1× Lottes and N64 4× royale (`vbench.sh`;
the layer's output is kept for each case).

- **The unmodified build reports 2,082 findings, and so does each lever, message for message.** There are 2,080
  `SYNC-HAZARD-READ-AFTER-WRITE` at `vkCmdDraw`, in royale and Mega Bezel, and two
  `VUID-RuntimeSpirv-OpEntryPoint-08743` at Mega Bezel's pipeline creation. Neither lever adds or removes one.
- These findings are production's, not this work's. §7.4's silent run missed them because it ran the pack presets
  under `dotnet test`, where the log file came back empty for every run of more than one test class. A run of the
  royale tests alone logged 150 of them. Why the larger runs lose the output was not established.
- The read-after-write is a pass sampling an image whose `vkCmdEndRenderPass` layout transition has no dependency on
  the next pass's fragment stage. `SlangChain`'s render passes declare no subpass dependency, so only the implicit
  external one applies, and it covers only the top and bottom of the pipe. That cause is read from the layer's
  message and the render pass's creation; no fix has tested it. On RADV it shows in no picture compared here. It is
  recorded, not fixed (§9.8). *(2026-09-24: fixed and the cause demonstrated in §10. The argued cause held: a
  subpass dependency to `EXTERNAL` takes the hazards to zero and removing it brings back every one. The
  `08743`s were a separate cause, two Mega Bezel shaders' unwritten and unread inputs, §10.3. The run of more than one
  test class losing the log is sidestepped rather than explained: the tests now take the layer's findings in
  process, §10.4.)*

**Positive controls.** Each ran through the same six cases:

- The rows image was written and blitted in `GENERAL`, with no barrier between. The layer reported 52
  `SYNC-HAZARD-READ-AFTER-WRITE` at `vkCmdBlitImage`, 26 in each N64 case.
- Queue access, which needs external synchronisation, was injected into `CreateBuffer`, which the parallel builders
  call at once. The layer reported 1,313 `UNASSIGNED-Threading-MultipleThreads-Write`. The clean tree, with parallel
  builds, has none.

So the layer sees both a missing transfer barrier and a race among the builders, and its silence about both in the
built tree is evidence.

**Negative results on the way.**

- The first positive control ran under `dotnet test` with the log file and reported nothing. §7.4's own control, run
  again the same way, reported nothing too. So the bench, with one process for each case, was used instead.
- Once, a restored source file kept an older timestamp, and the incremental build that followed kept the mutant. The
  runner now touches every file it restores.

**Mutants.** Each was applied alone, built and run against the Serenity tests (`mutants.log`):

| Mutant | Caught by |
|---|---|
| A readback an image holds is written again | `An_image_still_held_keeps_its_picture_…` |
| The image is made without marking its readback held | the test host crashes (use after free) |
| A disposed chain frees a readback an image still holds | the test host crashes |
| Rows blitted with linear filtering | seven tests, the N64 ones among them |
| The blit one row short | the same seven |
| The upload reshaped by height alone, not by repeat | `A_change_of_repeat_at_the_same_height_…` |
| The source image taken before the preset was built is kept | `A_preset_that_draws_takes_no_source_image_…` |
| The source copied under a preset as before | the same |
| The runner renders before letting its last image go | **survived** the first run; caught by `A_running_preset_draws_every_frame_from_one_readback_…`, written for it |
| The readback's host-read barrier removed | **survives** (below) |
| The key without the stage's text, the stage, the compiler's identity or the file name | `Every_input_to_a_compile_is_in_its_key`, and in turn the edited include, the two stages kept apart, another compiler's row, the second build's counts |
| The key's parts not length-prefixed | **survived** the first run, because the test's colliding pair was not two neighbouring parts of the key; caught once it was |
| The compile options (where defines would be) left out of the identity | `Every_input_to_a_compile_is_in_its_key` |
| A row served without its digest checked | `A_damaged_row_is_compiled_again_and_never_returned` |
| A file that is not a database not set aside | `A_file_that_is_not_a_database_…` |
| A locked database retried for ever, or waited on thirty seconds a statement | `A_locked_database_is_compiled_around_…` |
| Eviction takes the most recent, a hit does not refresh, or there is no bound | `The_cache_keeps_to_its_bound_…` |
| A second writer of a stage fails instead of being ignored | `Two_builds_of_one_preset_at_once_…` |
| Parallel passes stored in the order they finish | the parallel-build picture tests and the concurrent-build test |
| A parallel build reports the last broken pass | `A_parallel_build_that_fails_says_what_a_serial_build_would` |

The host-read barrier's mutant survives because on this device's host-coherent, host-cached memory the writes are
visible without it. Neither a picture nor the layer, which does not track host access, can tell the difference. The
barrier stays because the memory model requires it. **No test here would catch its removal on a device that needed
it.**

**The concurrent build** (`Two_builds_of_one_preset_at_once_neither_damage_nor_duplicate_the_cache`). Two caches over
one file, as two processes would have, build royale at once with four builders each. Afterwards:

- the file passes `PRAGMA integrity_check`;
- it holds exactly one row for each distinct stage (24);
- the two caches' stores sum to 24;
- both chains draw what an uncached serial build draws.

**Failure.**

- A file of random bytes is set aside, and the build goes on.
- A row with one bit flipped is deleted and compiled again.
- A database held under an exclusive lock cannot be opened, and every stage compiles.
- A database whose writes are held costs two skipped stores; the third turns the cache off. That took 3.5 s in all.

**The suite's own presets** are cached in its scratch directory (`home/tmp/WiseMan/`), set by a module initializer, so
no test writes to the player's `home/Shaders/`.

### 9.8 What is not done

- **Neither lever has run on the handheld.** The parent session measures it there. The desktop figures match the
  prototype's, which the handheld did measure (§8.4).
- **Nothing ran in a real window.** As in §8.9, the bench stands in for Mistress's render thread. §9.1's
  measurement of what Skia does with a raster image was on the bench's EGL context (the same radeonsi driver), not in
  Avalonia's own GL context in a window. If a window's context behaves differently, the held flag is what keeps a
  deferred upload safe.
- **Levers 3 and 4** (interop, and two frames in flight) were not built; they were not approved. Most of a light
  preset's remaining 1.8 ms at 1080p is what interop would remove.
- ~~**The chain's own synchronisation hazard is not fixed** (§9.7).~~ *Retired 2026-09-24: fixed in §10, with the
  picture comparison this bullet asked for (every bench case byte-identical, §10.5).* `SlangChain`'s render passes had
  no subpass dependency that made a pass's output visible to the next pass's fragment shader, and the layer reported
  this in royale and Mega Bezel in the unmodified build. It predated this work.
- **`Bind`'s per-frame allocations** (15–118 KB a frame for multi-pass presets) and the one gen2 collection left in
  N64 4× royale are not addressed. §8.5.5 bounded what caching descriptors could save.
- **The host-read barrier is untested** where it would matter (§9.7).
- **The cache is not pruned by pack.** Rows for a pack version no longer installed are left to the LRU bound; they
  are not deleted when the pack is replaced. Nor is the cache shared across machines or filled ahead of time: the
  first build of each preset still compiles, in parallel.
- **A `VkPipelineCache` is still not built.** §8.6 measured it at zero on drivers that have a disk cache.
- **The Shaders window's slider virtualisation** (§8.8's fifth lever) is out of scope. This lever makes the window's
  preset loads faster; its 944-row build is unchanged.

### 9.9 The handheld, after both levers (2026-09-24)

Measured by the parent session after the merge (9269ec9): the research's production bench (`deck-out/base`,
57937b2) against a bench built from WiseMan with both levers (`deck-out/after`), the same case lines, three rounds
with the order alternated, 300 frames each, the picture in the panel's 1920×1200. **On the charger this time**
(`power_supply` online 1), where §8.4's run was on battery, so the base figures here are not §8.4's and each lever is
read only against the base of this run. Render thread, median of the three rounds' medians:

| Case | Before (ms) | After (ms) | gen2 per 300 frames, before → after |
|---|---|---|---|
| N64 4× (2560×960 ×2), `crt-lottes` | 13.75 | **8.09** | 93–100 → 0 |
| N64 1× (640×240 ×2), `crt-royale` | 7.43 | 5.48 | 108–111 → 0 |
| SNES, `crt-lottes` | 7.23 | 5.70 | 150 → 0 |
| SNES, `crt-royale` | 6.01 | 4.54 | 75 → 0 |
| SNES, Mega Bezel SMOOTH-ADV | 14.37 | 13.18 | 43–46 → 0 |

The N64 4× case, the one §8.4 found missing the frame, lands where the prototype put it (8.22 ms in §8.4's lever run),
and every case's full collections are gone. The gains are smaller than the desktop's for Mega Bezel (−1.2 ms against
−2.3) because on this device its passes, not its transfers, are most of its frame. Raw results:
`~/.cache/emusen/probe/shaders/handheld/results-handheld-recheck.txt`; the bench host is
`~/.cache/emusen/probe/shaders/shaderbench-after/` built with `SerenityRoot` at the main tree. Not measured: preset
load times on the device after lever 2, and a run on battery.

## 10. The chain's synchronisation, and the flicker under CRT presets (2026-09-24)

§9.7 recorded a synchronisation hazard in `SlangChain` and argued its cause. The user asked whether it explained a
flicker they had seen under the CRT presets. This section reproduces the hazard, classifies it, demonstrates its
cause and fixes it. It then takes up the separate question of what makes a preset flicker here, and the answer to
that is mostly not the hazard. Scripts, logs and raw results are in `~/.cache/emusen/probe/shaders/sync/`. Its
`PREDICTIONS.md` holds the predictions, each written before the measurement it predicts.

### 10.1 What the layer reported, classified

The bench (§8.1) ran under `VK_LAYER_KHRONOS_validation` with synchronisation validation, the shader-access heuristic
and no message limit, one process per case (`vbench.sh`). It covered seventeen cases: §8.3's thirteen slang cases,
plus `crt-guest-advanced-ntsc`, `phosphor-persistence` (feedback), `mix_frames` (history) and `bob-deinterlacing`.
Each run draws 26 frames. The unmodified build (`9269ec9`) reported **5,408 `SYNC-HAZARD-READ-AFTER-WRITE` at
`vkCmdDraw` and 5 `VUID-RuntimeSpirv-OpEntryPoint-08743`**. These are the messages §9.7 counted over its six cases.

To say which pass and which image each message is about, an uncommitted copy of the chain printed, for every pass,
its pipeline, its descriptor set, its output views and what each binding resolves to. `classify.py` joined that with
the handles in the layer's messages (`classified-base.txt`). All 5,408 are the same kind of finding:

- **Read-after-write**, never write-after-read or write-after-write.
- The reader is a **fragment shader** in pass *j*, sampling the output of an earlier pass *i* < *j* **of the same
  frame**, by `Source`, by `PassOutputN` or by alias.
- The prior access is **the layout transition that `vkCmdEndRenderPass` performs** (colour attachment to
  shader-read) at the end of pass *i*.
- The layer reports each (pass, binding) once a frame. That gives 13 a frame in guest-advanced, 15 in royale, 7 in
  Mega Bezel POTATO, 50 in SMOOTH-ADV, 26 in guest-advanced-ntsc and 2 in phosphor-persistence. Lottes,
  `lcd-grid-v2`, `mix_frames` and `bob-deinterlacing` report nothing: none has a pass that reads another pass's
  output from the same frame.

What was **not** reported is as informative:

- **Feedback** (`PassFeedbackN`, alias + `Feedback`) is never flagged. It reads last frame's image, written in an
  earlier submission that the host waited for.
  - A pass whose output is read as feedback keeps two images and swaps them after each frame (§7.4).
  - So in the classification that pass's current output appears as "image A" on odd frames and "image B" on even
    ones. Both are reads of the same frame's output.
- **History** (`Original`, `OriginalHistoryN`) is never flagged. `Advance` uploads it in its own submission.
- **Lookup images** are never flagged. They are uploaded once, when the preset is built.
- **An output with a mip chain** is never flagged, because `GenerateMipmaps` follows the pass with `Transition`'s
  all-commands barrier. That is why the reads of guest-advanced's mipmapped output are absent.
- **The readback and the row blit** (§9.1, §9.2) are never flagged. The same all-commands barriers cover them, and
  §9.7's positive control showed that the layer reports a missing one there.

### 10.2 The cause, and the fix

Each pass's render pass declared no `VkSubpassDependency`, so Vulkan supplied an implicit one at each end of the
subpass. The implicit dependency out of the subpass has the colour writes and the final layout transition as its
source, and `BOTTOM_OF_PIPE` with no access as its destination. That destination orders nothing that comes later and
makes nothing visible to it. The next pass's fragment shader may therefore sample the image before the transition to
shader-read has finished, or before the colour writes are visible to its caches. §9.7 read this cause from the
messages but did not test it.

**The fix** is one dependency on every pass's render pass, from subpass 0 to `VK_SUBPASS_EXTERNAL`:

| | Stages | Access |
|---|---|---|
| source | colour-attachment output | colour-attachment write |
| destination | vertex shader, fragment shader | shader read |

The destination names the vertex stage because libretro lets a vertex shader sample a pass's output, and the
descriptor layouts already give every sampler to both stages. A sweep of every `.slang` in the pack found no preset
that does this, so a test preset does (§10.5).

No dependency *into* the subpass was added, because there is no write-after-read to order:

- Within a frame, nothing reads a pass's output before the pass writes it.
- Across frames, the host waits for each submission before it records the next.

**How RetroArch does it** (`gfx/drivers_shader/shader_vulkan.cpp`, read, nothing copied). Its render passes declare
no dependency either. After each pass's `vkCmdEndRenderPass` it records an explicit image barrier, from
colour-attachment output and write to fragment shader and read, and moves the image to shader-read-only itself.

- The masks are the ones chosen here, except that RetroArch's barrier names only the fragment stage.
- A barrier after the pass and a dependency on the pass order the same accesses. The dependency was kept because it
  needs no layout bookkeeping outside the render pass.

**Why no picture on this machine ever showed it** is argued, not demonstrated:

- RADV turns the source half of the implicit dependency into a wait for the pixel shaders and a flush of the colour
  caches at the end of every render pass, whatever the destination.
- The image being sampled was written only in this frame, after the submission began with its caches invalidated, so
  no stale line can be read.
- The hazard could therefore become visible on a driver that does less for that source half, such as another
  vendor's or a tiled mobile GPU's. This machine cannot test that.

### 10.3 `VUID-RuntimeSpirv-OpEntryPoint-08743`

The rule is that every user-defined input of a stage must be written as an output by the stage before it, location
for location and component for component. Printing each pass's module handles beside the layer's messages traced the
five messages to three shaders:

- Mega Bezel POTATO, `hsm-crt-dariusg-gdv-mini.inc`. The fragment stage declares
  `layout(location = 1) in float maskFade;`, and the vertex stage has no output at location 1.
- Mega Bezel SMOOTH-ADV, `bezel-images.inc`, included by `bezel-images-under-crt` and `bezel-images-over-crt`. The
  fragment stage declares `layout(location = 8) in vec3 BEZEL_FRAME_ORIGINAL_COLOR_RGB;`, and the vertex stage writes
  only locations 6 and 7.

**No instruction in either shader reads the variable.** It is a declaration left behind in the pack.

- RetroArch compiles the same text with glslang and builds the same interface.
- An input like this would read an undefined value, but since nothing reads it, nothing on screen depends on it.
- The pipeline is nonetheless invalid, and a driver is entitled to reject it.

**The fix** is `SpirvReflection.WithoutUnreadInputs`. It runs after a fragment stage is compiled or served from the
cache.

- It leaves out of the entry point's interface list any `Input` variable that no instruction inside a function refers
  to, and corrects the instruction's word count.
- This is valid SPIR-V. SPIR-V 1.3, which Vulkan 1.1 uses, requires the interface to list the inputs the entry point
  *uses*, and an unused input need not be listed.
- It runs after the cache, so the cache's rows stay Shaderc's output and no key changes.
- A module with nothing to drop is returned as the same array.

**The test for "read" is deliberately one-sided.** Any word in a function body equal to the variable's id counts as a
use, so a literal that happens to match keeps an input but never drops one. The first version scanned the whole
module, and in the test it kept `AlsoUnread`, whose id matched a literal among the declarations. That is why the scan
starts at the first `OpFunction`.

**What it does not fix** is an input that *is* read but never written. That would be a real shader defect, with an
undefined value on screen. No bench case has one, and the layer would still report it, which is right.

### 10.4 The layer in process, for the tests

§9.7 found that the layer's log file came back empty under `dotnet test` whenever more than one test class ran. The
tests now avoid the file rather than explain that. `SlangVulkan.TryCreate` takes an internal sink, and with one it
loads the layer:

- It enables `VK_LAYER_KHRONOS_validation`, configured through `VK_EXT_layer_settings`: `validate_sync` and
  `syncval_shader_accesses_heuristic` on, `enable_message_limit` off.
- A `VK_EXT_debug_utils` messenger hands every validation and performance message to the sink.
- The public `TryCreate` passes no sink, so no frontend loads the layer.

**The messenger must be made after the instance exists.** The first version chained it into `VkInstanceCreateInfo`.
It received the loader's messages during instance creation and nothing afterwards, so the first tests passed while the
layer reported to no one. The messenger is now made with `vkCreateDebugUtilsMessengerEXT` once the instance exists.

**Silence proves something only if the layer is listening.** After each run the tests create a zero-sized buffer,
which is `VUID-VkBufferCreateInfo-size-00912`, and assert that the sink received that message. This check is what
exposed the messenger mistake above.

**Where the layer is not installed**, instance creation fails and the tests return, as every GPU test here does
without a device. On such a machine these tests say nothing.

### 10.5 The evidence

**Validation after the fix.** The same seventeen cases under the same settings gave **zero findings on every case**,
N64 4× and 4K SMOOTH-ADV included (`validation-fix/`).

**Positive controls.** Each control was the fixed tree with one change, run over the same seventeen cases:

| Control | Findings |
|---|---|
| Subpass dependency removed (`DependencyCount = 0`) | 5,408 `SYNC-HAZARD-READ-AFTER-WRITE`, exactly the unmodified build's count |
| Unread inputs kept | 5 `VUID-…-08743`, and no hazard |

So the dependency removes every hazard and nothing else does, and the interface change removes every VUID and
nothing else does.

**Pictures.** The bench hashes three frames per run after the measured ones.

- Against `9269ec9`, interleaved, three rounds of all thirteen slang cases: **13 of 13 cases byte-identical, on
  every frame of every round** (`summary-time.md`).
- The seventeen validation runs matched frame for frame before and after.
- On the N64 frame the still bench (§10.6) gave the same sequence pattern for all 101 `crt/` presets before and
  after. For the three `zfast` presets this holds once alpha is excluded; the first sweep hashed their undefined
  alpha.
- **So nothing the fix changes is visible on RADV.** That was prediction P2. It fits §10.2's argument, and it means
  the fix changes no picture a player has seen on this hardware.

**Cost.** The prediction was a change within ±0.05 ms for one-pass presets and at most +0.10 ms for royale and Mega
Bezel. The table gives the render thread's time in ms: the median of three rounds' medians, interleaved under the
bench lock (`results-time.txt`).

| Case | `9269ec9` | Fixed | Change |
|---|---|---|---|
| SNES `crt-lottes` 1080p | 1.79 | 1.80 | +0.01 |
| SNES `crt-guest-advanced` | 1.87 | 1.90 | +0.02 |
| SNES `crt-royale` | 1.74 | 1.76 | +0.02 |
| SNES Mega Bezel POTATO | 1.84 | 1.85 | +0.01 |
| SNES Mega Bezel SMOOTH-ADV | 3.92 | 3.91 | −0.01 |
| GB `lcd-grid-v2` | 1.33 | 1.34 | +0.01 |
| N64 1× `crt-lottes` | 1.98 | 1.97 | −0.01 |
| N64 1× `crt-royale` | 1.94 | 1.96 | +0.01 |
| N64 4× `crt-lottes` | 2.86 | 2.86 | −0.01 |
| N64 4× `crt-royale` | 3.57 | 3.55 | −0.02 |
| SNES 4K `crt-lottes` | 5.30 | 5.29 | −0.01 |
| SNES 4K `crt-royale` | 4.45 | 4.42 | −0.03 |
| SNES 4K Mega Bezel SMOOTH-ADV | 9.17 | 9.17 | +0.01 |

- Every change is within ±0.03 ms, the size of the spread from round to round. The prediction held.
- The passes' GPU time (`chain.gpu.passes`) moved by at most 0.04 ms, and by 0.07 ms in one case: N64 1× Lottes.
  Lottes is a one-pass preset, so that fall cannot come from a dependency between passes, and it is read as noise.
- The fix costs nothing measurable here, which fits §10.2's argument that RADV already did the waiting.

**Mutants.** Each mutant was applied alone to the worktree, built, and run against `SlangSyncTests`, `SlangChainTests`
and `SlangReadbackTests` with the pack (`mutants.py`, `mutants.log`). "The four validation tests" below are those for
the test preset, royale, guest-advanced and POTATO.

| Mutant | Caught by |
|---|---|
| No subpass dependency | the four validation tests |
| The dependency's destination without the fragment stage | the four validation tests |
| The destination without the vertex stage | **survived** the first run, because no preset sampled in a vertex shader. Caught by the test preset once its fourth pass did (§10.2) |
| No destination access (an execution dependency only) | the four validation tests |
| From the top of the pipe, with no source access | the four validation tests |
| Unread inputs kept | the validation tests for the test preset and for POTATO |
| Every input dropped, read or not | a test-host crash: the driver was given a fragment stage without its `vTexCoord` |
| The whole module scanned for reads | `An_input_no_instruction_reads_…`, and the same two validation tests |
| The entry point's word count not corrected | a test-host crash |

Both crashes count as caught, as §9.7's did, but they were caught by a crash, not by an assertion.

### 10.6 What makes a preset flicker here

The hazard changes no picture on this machine (§10.5), so it cannot be what the user saw on it. The user's machines
are this desktop's RX 6800 and a Legion Go S, both on RADV, and §10.2's argument covers both. The question therefore
became what else could make a preset's picture unstable from frame to frame. The candidates were:

- feedback swapped at the wrong time;
- history uninitialised or wrong;
- `FrameCount` or `FrameDirection` wrong;
- a frame run again, or not run, when it should be;
- the lent readback overwritten;
- uninitialised images.

**The still bench** (`still-base/Program.cs`, a probe, not committed) tests these by builds and hashes:

- It builds a preset twice and gives each chain the same still frame N times, as a running game on a static screen
  would.
- It hashes each picture with its alpha set opaque. The frame control draws the image opaque, so alpha is never seen.
- It reports how many distinct pictures there were, their pattern, the steady state's period, and whether the two
  chains agree frame for frame.
- It ran over all 101 presets in the pack's `crt/`, on two frames: a 256×224 SNES-shaped frame, and a 640×240
  N64-shaped frame with its rows repeated twice, as the frame control then sent it (`still-base-crt.txt`).

**What it found, sorted:**

- **Most presets hold still.** One picture for every frame, and both chains agree.
- **Some presets animate by design.** They change every frame, both chains agree, and each uses `FrameCount` for
  grain, noise, rolling scan or jitter:
  - `newpixie-crt`, `crt-mattias`, `crt-pocket`, `metacrt`, `monoCRT`, `simple-crt`, `gizmo-crt`,
    `crt-beans-rgb`/`-vga` and `crt-maximus-royale` change every frame.
  - `crt-yah` changes every five frames with a period of a hundred, because its noise runs at 12 Hz.
  - `crt-1tap-bloom` settles after 53 frames. Its moving average is eye adaptation, through feedback.
- **The NTSC and composite presets alternate on both frames**: `crt-hyllian-ntsc`, `-ntsc-rainbow`,
  `crt-consumer-1w-ntsc-XL` and `crtsim`. This is the colour subcarrier's phase alternating by `FrameCount`, which is
  what those shaders emulate.
- **`gizmo-slotmask-crt` is the one preset whose two chains disagree.** This is a pack defect, recorded and not
  fixed:
  - About 260 of 1.2 million pixels differ, by up to 216 levels, all at the edge of its curved screen.
  - The shader calls `fwidth` after an early `return` for pixels outside the curve. So it takes derivatives in
    non-uniform control flow, where they are undefined.
  - It sparkles at the border in RetroArch too, for the same reason, and the fix changes nothing about it.
  - It refutes prediction P5 for one preset, for a reason unrelated to the hazard.
- **Some presets leave alpha undefined**: `zfast-crt`, `simple-crt`, `crt-geom-deluxe` and Mega Bezel. The last pass
  writes only `.rgb`, so alpha differs between frames and between chains. It is never visible, because the image is
  drawn opaque.
- **On the N64 frame only, 24 presets alternate between two pictures every frame.** On the SNES frame the same
  presets hold still. They are royale and all its variants, the guest-advanced family, `crt-geom`, `crt-geom-mini`,
  `fake-crt-geom`, `crt-geom-deluxe`, `crt-consumer`, `crt-Cyclon`, `crt-beans-fast`, `crt-easymode-halation`,
  `crt-nobody`, `crt-interlaced-halation` and `zfast-crt-composite`.

**The cause of the N64 alternation.** These presets emulate interlacing when the picture is tall enough to be an
interlaced one:

- Royale does so for 288.5 to 576.5 lines (`interlace_detect_toggle`, on by default).
- Guest-advanced does so at 375 lines and up (`inter` 375, `interm` 1).
- Each shows one field's lines, and moves to the other field as `FrameCount`'s parity changes.

The frame control sent a progressive N64 field, 240 rows, with its rows repeated to 480 before the preset saw it
(§7.4, §9.2). So the preset took every 240p game for a 480i one. A still screen then alternates between two pictures
at half the rate new frames arrive. Mars offers a new picture only when the game draws one (Mars_Video.md §2.8), so
on a 20 or 30 fps game the alternation runs at 10 or 15 Hz, unevenly paced. That is a visible, irregular line flicker.

**Against RetroArch** (read from `~/Projects/retroarch-reference` and `~/Projects/parallel-n64-reference`).
parallel-n64 hands `video_cb` 240 rows for a progressive field, and 480 only for an interlaced one:

- under angrylion (`vi.c`), the height is `480 >> !serrate`;
- under parallel-rdp, it is the scan-out's height, `(480 >> !serrate) × scale`.

So RetroArch's royale does not interlace a 240p N64 game, and EmuSen's did. The §7.4 premise, "a slang pass sees the
picture as the screen would", was wrong for a CRT preset. The screen it emulates receives 240 lines per field. The
repeat is this frontend's way of filling a progressive display, not part of the signal.

**The fix.** `SlangRunner` now advances the chain with the rows once.

- The destination rectangle already carries the repeat (§2.7), so the shape on screen is unchanged.
- The chain keeps its repeat argument, and §9.2's blit stays for a caller that wants it. The frame control no longer
  does.
- SNES, GB and interlaced N64 frames arrive with a repeat of one and are untouched.
- At Mars's 4× the preset now sees 960 rows, which is what parallel-rdp's 4× upscale gives RetroArch. At 960 rows
  guest-advanced still emulates interlacing (960 ≥ 375), in both frontends, and royale does not (960 is outside its
  range).

**The evidence for the fix:**

- **The still bench after the fix** (`still-fix-n64-rows-once.txt` against `-rows-twice.txt`, the same tree). On the
  N64 frame 5 presets alternate instead of 24. Four are the NTSC and composite presets, which alternate on the SNES
  frame too. The fifth is `crt-guest-advanced-ntsc`, whose NTSC pass picks a two-phase pattern for a picture 640
  wide.
- **The tests.** `A_still_progressive_frame_under_royale_draws_the_same_picture_every_frame` shows the defect and the
  fix in one test:
  - The chain, given the frame as 480 rows, alternates: frames 0 and 1 differ, and frames 0 and 2 agree.
  - The runner, given the same frame, draws one picture four times.
- `A_progressive_frame_reaches_the_preset_with_its_rows_once` reads `OriginalSize` back through a preset.
- A mutant runner that repeats the rows as before fails both tests.
- **Pictures.** The rows-once tree against the fixed tree, interleaved, three rounds of the thirteen slang cases
  (`results-rows.txt`, `summary-rows.md`):
  - The nine SNES, GB and 4K cases are byte-identical.
  - The four N64 cases differ, which is intended: the preset now works on 240 rows, or 960 at 4×, where it had 480
    or 1,920. Each build drew the same three pictures in every round.
  - Looked at (`dump/royale-n64-compare.png`): with the rows repeated, royale's picture moves up and down by a
    line between consecutive frames of a still screen, which is the alternation the bench counted. With the rows
    once, the frames are identical, and the preset draws a scanline structure for 240 lines where it drew one for
    480.
  - The validation layer stays at zero on all seventeen cases.
- **Cost.** It was predicted at −0.05 to −0.15 ms at N64 1×, −0.2 to −0.5 at 4× Lottes and −0.5 to −1.0 at 4×
  royale. Measured, in ms:

  | Case | Fixed | Rows once | Change |
  |---|---|---|---|
  | N64 1× Lottes | 1.96 | 1.98 | +0.02 |
  | N64 1× royale | 1.94 | 1.95 | +0.01 |
  | N64 4× Lottes | 2.85 | 2.80 | −0.05 |
  | N64 4× royale | 3.60 | 3.23 | −0.37 |

  **All three predictions are refuted, and all in the same direction: the gain is smaller than predicted.**
  - `Advance` fell by only 0.02 to 0.06 ms. The blit to the doubled height was cheap, and the upload of the rows,
    which remains, is most of `Advance`.
  - At 1× the passes' work did not measurably shrink.
  - At 4× royale's GPU passes fell from 1.17 to 0.86 ms, because its source-scaled passes now run at half the
    height.
  - The nine unchanged cases moved by −0.06 to +0.03 ms. That is wider than the ±0.03 predicted (P7) for cases the
    change cannot touch, and it is the noise of this pair of runs.

**Other divergences from RetroArch, found on the way.** None of them makes a flicker here.

- **`FrameCount` counts pictures drawn, not console frames. This is not fixed.**
  - RetroArch increments it at every video frame. That includes a core's duplicated frame (the chain runs again,
    history is pushed, feedback swaps), frames while paused (the cached frame is drawn at the display's rate), and
    even fast-forward frames it skips drawing.
  - EmuSen advances it once for each new offer the frame control draws.
  - So on an N64 game at 20 fps, a `FrameCount` animation runs at a third of RetroArch's rate, and a paused game's
    picture holds still where RetroArch's keeps animating.
  - For a genuinely interlaced (480i) N64 picture, the field alternation still follows the game's frame rate.
  - A fix needs the frontend to pass a console frame number with each offer. Whether a paused picture should keep
    animating is the user's choice.
- **RetroArch defines `_HAS_ORIGINALASPECT_UNIFORMS`, `_HAS_FRAMETIME_UNIFORMS` and `_HAS_SENSOR_UNIFORMS`** in every
  stage it compiles (`glslang_util.c`). EmuSen passes Shaderc no defines (§9.4), so a shader written for them takes its
  fallback. For instance, `crt-yah`'s frame-rate compensation assumes 60 fps instead of reading `FrameTimeDelta`.
  That makes nothing unstable, but it is a difference.
- **These agree with RetroArch, and none showed anything in the still bench:**
  - `FrameDirection` is 1, as RetroArch's is when not rewinding.
  - Feedback swaps after the frame's passes.
  - History and feedback start cleared.
  - The lent readback is never written while an image holds it (§9.1's tests).

### 10.7 Does this explain the user's flicker?

**The synchronisation hazard does not, on the user's hardware.** It is real and it is fixed. It broke the API's rules
and could have shown on another driver. But on RADV its fix changes no picture, in any bench case or in any of the 101
CRT presets. A defect whose fix changes no picture cannot be what the user saw.

**The interlace emulation on N64 frames very probably does, if the flicker was on an N64 game** under royale,
guest-advanced or any of the other presets listed in §10.6:

- On a still screen it shows as line flicker over the whole picture at 10 to 30 Hz, unevenly paced by the game's
  frame rate. That matches "weird flickering with the CRT shaders".
- It is gone after §10.6's fix.

Under a SNES or Game Boy game none of this applies. What still changes from frame to frame there is presets
animating on purpose (grain, noise, NTSC phase) and one pack shader's border sparkle (`gizmo-slotmask-crt`).

Which game and preset the user saw the flicker under was not established. If the flicker persists, that is the
question to ask.

### 10.8 What this does not cover

- **No other vendor's driver was run.** §10.2's reason the hazard never showed is argued from RADV's behaviour, and
  the claim that it could show elsewhere is untested. The fix is what the specification requires either way.
- **Nothing ran in a real window**, as in §9.8. The flicker findings are the chain's and the runner's pictures,
  hashed. The frame control's own redraw pacing was not observed in Mistress.
- **The handheld was not run.** The parent session runs it.
- **`FrameCount`'s rate** (§10.6) is not fixed. Nor is RetroArch's behaviour of running the chain again on a
  duplicated or paused frame.
- **`gizmo-slotmask-crt`'s border sparkle and the undefined alphas** belong to the pack and are not worked around.
- **A fragment input that is read but never written** would still be reported by the layer, and still be undefined
  on screen (§10.3).
- **The validation tests say nothing where the layer is not installed** (§10.4). Where it is installed, they check
  that it is listening.
- **The still bench swept only the pack's `crt/` presets.** Presets in the other categories (`bezel/`, `handheld/`,
  `ntsc/` and the rest) were run only where the bench cases include them.
