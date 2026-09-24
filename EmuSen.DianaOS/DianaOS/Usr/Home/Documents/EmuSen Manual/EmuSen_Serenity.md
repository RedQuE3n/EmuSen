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
| P7 | Handheld: GPU passes 3–5× the desktop's; Mega Bezel over 16.7 ms | Not yet tested (§8.4) |
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

**Not yet measured.** The Legion Go S (Z1 Extreme, RDNA 3 integrated, 1920×1200 panel, SteamOS) was not reachable
from this session with a method the session was allowed to use. Everything to run there is built: self-contained
linux-x64 publishes of both benches (natives asking for glibc 2.27 at most, the device has 2.41) in
`~/.cache/emusen/probe/shaders/deck-out/{base,proto}`, the case lists in `~/.cache/emusen/probe/shaders/deck/`
(SNES, Game Boy and N64 letterboxed into 1920×1200, the levers, the build times), and `deck-run.sh`, which runs all
of them into `~/emusen-bench/shaders/results-*.txt` there. P7 and these predictions for the device stand untested:

- **H1.** The transfer stages cost more there, not less: one memory serves both processors, so the readback, the two
  host copies and Skia's upload compete with the GPU's own bandwidth; Lottes at 1371×1200 over 4 ms.
- **H2.** The passes are 3–5× the RX 6800's; Mega Bezel SMOOTH-ADV's alone over 8 ms, and with the path's fixed costs
  the frame over 16.7 ms.
- **H3.** The levers' order does not change, but interop gains more than on the desktop, since the GPU work it removes
  (readback and upload, 0.8 ms a frame here) is taken from a GPU that is then the bound.

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

**The recommended order** is the table's. The first two are small, exact (identical pictures), cost no latency and
between them remove most of a light preset's avoidable frame cost and nearly all of a heavy preset's build time. The
third is the one that makes a slang preset cost what a built-in filter costs, and it is the one the handheld is most
likely to need (H3); it should be decided on §8.4's numbers once they exist, as Mars_Gpu.md §16 said of its own
interop. The fourth trades latency for render-thread time and should come last, if at all, and then as a setting.

### 8.9 What this does not cover

- **The handheld** (§8.4): nothing was measured there.
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
