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

---

## 3. The built-in shaders

Two single-pass effects, `Scanlines` and `Crt`, as SkSL source embedded in `BuiltInShaders`. `ShaderEffect.None` is always the fallback and is bit-identical to having no shader pipeline at all.

They are faithful ports of the original Raylib/GLSL versions, not a redesign — the port was part of moving off Raylib, and doing it as a translation kept the two changes separable. They stay embedded as C# string constants for the same reason the GLSL predecessors were: no "did the build actually copy this file" failure mode while the pipeline itself is still being proven out. `BuiltInShaders` is `internal`, an implementation detail of `GameFrameControl` rather than public API; `AssemblyInfo.cs` opens it to `EmuSen.WiseMan` alone so the SkSL source can be asserted directly.

**What changes when translating GLSL to SkSL:**

- **Sampling another shader.** SkSL's `uniform shader image` plus `image.eval(coord)` is the direct equivalent of GLSL's `sampler2D` / `texture(texture0, fragTexCoord)` pair. SkSL has no `sampler2D`.
- **`coord` is in destination pixel space, not a normalized UV.** GLSL's `fragTexCoord` arrived as 0–1; SkSL's `coord` arrives in the destination's local pixel coordinates. The scanline row check uses it directly (`floor(coord.y)`), which is exactly what makes the effect run at output resolution and stay crisp at any window size. The vignette still needs the original 0–1 space its math was written against, so it divides by `outputSize` first to recover it.
- **The `image` child is sampled in the *source* image's native pixel coordinates**, not stretched to any destination rect. This is why §2.2's local `SKMatrix.CreateScale(scaleX, scaleY)` matters: without it the row and vignette math would operate over the frame's native-resolution top-left corner of the upscaled output, shading a small patch instead of the picture. The matrix maps `coord` back onto the native source, so the effect runs in real output-pixel space with no intermediate render target.

The exact darkening factors are part of the contract, not incidental: `Scanlines` leaves even output rows at full brightness and multiplies odd rows by **0.78**; `Crt` uses **0.72** and adds a vignette of `1 - dot(centered, centered) * 0.55`. `EmuSen.WiseMan`'s `Scanlines_darkens_odd_output_rows_by_the_exact_documented_factor` renders a solid-color frame and checks that arithmetic precisely, which is what proved the local-matrix rewrite correct rather than merely different.

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
