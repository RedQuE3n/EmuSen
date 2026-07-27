using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Rendering.SceneGraph;
using Avalonia.Skia;
using SkiaSharp;
using EmuSen.Graphics;
using EmuSen.Serenity.Shaders;

namespace EmuSen.Serenity
{
    // Renders the emulator's raw RGBA8888 frame buffer straight through
    // Avalonia's own Skia backend, with an optional single-pass shader
    // effect (see ShaderEffect) - the Avalonia-native replacement for
    // FramePresenter's old Raylib render-texture/GLSL pipeline (see git
    // history). ICustomDrawOperation is the documented way to get direct
    // SKCanvas access inside an Avalonia Render() call - see Render()'s
    // own comment for exactly how.
    //
    // Deliberately does NOT reproduce the old fixed configured-resolution
    // (GraphicsSettings.WindowWidth/Height) composite canvas the Raylib
    // version letterboxed - that canvas existed specifically to give the
    // on-window debug overlay (VRAM sheet/CGRAM swatch/register text,
    // drawn at fixed pixel offsets around a small 2x-scaled game view) a
    // fixed layout to share with the game screen. That overlay is gone
    // (see Man pages/EmuSen_Debugging_Tools_Reference_v5.md's own revision
    // note on why), so letterboxing the actual game frame's own native
    // aspect ratio directly into the control's real bounds is what "show
    // me the game, scaled to fit the window" now means - the old
    // reference-canvas math would otherwise show the game tiny in one
    // corner with dead space filling the rest, a visible regression, not
    // a faithful port.
    public sealed class GameFrameControl : Control
    {
        private byte[]? _rgba;
        private int _frameWidth;
        private int _frameHeight;

        private readonly Dictionary<ShaderEffect, SKRuntimeEffect> _effects = new();

        // Named ActiveEffect, not Effect - Avalonia's own Visual base
        // class already has an unrelated Effect property (a compositor
        // bitmap effect like blur/drop-shadow), and hiding it would be
        // confusing even though it's legal.
        public ShaderEffect ActiveEffect { get; set; } = ShaderEffect.None;

        // Called once per presented frame (already on the UI thread - see
        // FramePresenter.Present()'s own dispatch). Just stores the latest
        // buffer and asks Avalonia to repaint; actual drawing happens in
        // Render() below, on Avalonia's own render pass.
        public void UpdateFrame(byte[] rgba, int width, int height)
        {
            _rgba = rgba;
            _frameWidth = width;
            _frameHeight = height;
            InvalidateVisual();
        }

        // Fit-preserving-aspect-ratio math: scales (sourceWidth,
        // sourceHeight) as large as possible inside (actualWidth,
        // actualHeight) while staying centered - the same shape of
        // computation FramePresenter's old Raylib letterbox used, just
        // against the real source/destination sizes instead of a fixed
        // reference canvas. Pure arithmetic, no Avalonia/Skia types, so
        // it's directly unit-testable.
        public static (double X, double Y, double Width, double Height) ComputeLetterboxRect(
            double sourceWidth, double sourceHeight, double actualWidth, double actualHeight)
        {
            if (sourceWidth <= 0 || sourceHeight <= 0 || actualWidth <= 0 || actualHeight <= 0)
            {
                return (0, 0, 0, 0);
            }

            double scale = Math.Min(actualWidth / sourceWidth, actualHeight / sourceHeight);
            double w = sourceWidth * scale;
            double h = sourceHeight * scale;
            return ((actualWidth - w) * 0.5, (actualHeight - h) * 0.5, w, h);
        }

        // Lazily compiles a built-in SkSL effect on first use - the same
        // "don't pay for shader compilation until it's actually selected"
        // reasoning the old GetShader(ShaderEffect) had. The compiled
        // SKRuntimeEffect is frame-invariant and cached here; the SKShader
        // built from it is NOT (it has to rebind each frame's own image),
        // so that part happens fresh in Render() below, not here.
        private SKRuntimeEffect GetEffect(ShaderEffect effect)
        {
            if (_effects.TryGetValue(effect, out SKRuntimeEffect? existing)) return existing;

            string sksl = effect switch
            {
                ShaderEffect.Scanlines => BuiltInShaders.ScanlinesSksl,
                ShaderEffect.Crt => BuiltInShaders.CrtSksl,
                _ => throw new ArgumentOutOfRangeException(nameof(effect), effect, "No shader source for this effect."),
            };

            SKRuntimeEffect compiled = SKRuntimeEffect.CreateShader(sksl, out string errors);
            if (compiled == null)
            {
                throw new InvalidOperationException($"Failed to compile the '{effect}' SkSL shader: {errors}");
            }

            _effects[effect] = compiled;
            return compiled;
        }

        public override void Render(DrawingContext context)
        {
            base.Render(context);
            if (_rgba == null || _frameWidth <= 0 || _frameHeight <= 0) return;

            context.Custom(new DrawOp(new Rect(Bounds.Size), this, _rgba, _frameWidth, _frameHeight, ActiveEffect));
        }

        // One custom draw operation per Render() call, carrying an
        // immutable snapshot of everything it needs - Avalonia may call
        // Render() again on the same instance before this one's Render()
        // actually runs, so this can't read back through mutable fields
        // on the control itself.
        private sealed class DrawOp : ICustomDrawOperation
        {
            private readonly GameFrameControl _owner;
            private readonly byte[] _rgba;
            private readonly int _width;
            private readonly int _height;
            private readonly ShaderEffect _effect;

            public DrawOp(Rect bounds, GameFrameControl owner, byte[] rgba, int width, int height, ShaderEffect effect)
            {
                Bounds = bounds;
                _owner = owner;
                _rgba = rgba;
                _width = width;
                _height = height;
                _effect = effect;
            }

            public Rect Bounds { get; }

            public bool HitTest(Point p) => Bounds.Contains(p);

            public bool Equals(ICustomDrawOperation? other) => false; // always re-render - this is a live video feed, never cacheable

            public void Dispose() { }

            public void Render(ImmediateDrawingContext context)
            {
                var feature = context.TryGetFeature<ISkiaSharpApiLeaseFeature>();
                if (feature == null) return; // non-Skia backend - nothing to draw through

                using ISkiaSharpApiLease lease = feature.Lease();
                SKCanvas canvas = lease.SkCanvas;

                var sourceInfo = new SKImageInfo(_width, _height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
                using SKImage sourceImage = SKImage.FromPixelCopy(sourceInfo, _rgba);

                // GraphicsSettings.BilinearFiltering - the same knob the
                // old Raylib TextureFilter.Bilinear/Point choice drove.
                SKSamplingOptions sampling = GraphicsSettings.BilinearFiltering
                    ? new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None)
                    : new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None);

                var (x, y, w, h) = ComputeLetterboxRect(_width, _height, Bounds.Width, Bounds.Height);
                int upscaledW = Math.Max(1, (int)Math.Round(w));
                int upscaledH = Math.Max(1, (int)Math.Round(h));
                var destRect = new SKRect((float)x, (float)y, (float)x + upscaledW, (float)y + upscaledH);

                // Confirmed fix for a real flicker bug (see Man pages/
                // EmuSen_Project_Overview_v2.md §2a): allocating/disposing
                // a fresh GPU-backed SKSurface every single frame (below)
                // was the cause, on both X11 and Wayland. Skip it entirely
                // when no shader is active - DrawImage scales straight to
                // destRect on its own.
                if (_effect == ShaderEffect.None)
                {
                    canvas.DrawImage(sourceImage, destRect, sampling);
                    return;
                }

                // Two stages, mirroring the old Raylib pipeline exactly:
                // (1) scale the native-resolution game frame up to the
                // final on-screen size first, into an offscreen surface;
                // (2) run the shader over THAT already-upscaled image, so
                // its row/vignette math operates in real output-pixel
                // space (see BuiltInShaders' own comment). A shader's
                // child "image" is sampled in the source image's own
                // native pixel coordinates, not stretched to any
                // destination rect - skipping this stage would only
                // shade the frame's native-resolution top-left corner of
                // the upscaled output, not the whole thing.
                var upscaledInfo = new SKImageInfo(upscaledW, upscaledH, SKColorType.Rgba8888, SKAlphaType.Unpremul);
                using SKSurface upscaleSurface = lease.GrContext != null
                    ? SKSurface.Create(lease.GrContext, budgeted: false, upscaledInfo)
                    : SKSurface.Create(upscaledInfo);
                upscaleSurface.Canvas.Clear(SKColors.Transparent);
                upscaleSurface.Canvas.DrawImage(sourceImage, new SKRect(0, 0, upscaledW, upscaledH), sampling);
                using SKImage upscaledImage = upscaleSurface.Snapshot();

                // Not `using` - see Man pages/EmuSen_Project_Overview_v2.md
                // §2a on why disposing this crashes the process.
                SKRuntimeEffect compiled = _owner.GetEffect(_effect);
                var builder = new SKRuntimeShaderBuilder(compiled);
                builder.Children["image"] = upscaledImage.ToShader(SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, sampling);
                builder.Uniforms["outputSize"] = new[] { (float)upscaledW, (float)upscaledH };

                using SKShader shader = builder.Build();
                using var paint = new SKPaint { Shader = shader };
                canvas.Save();
                canvas.Translate((float)x, (float)y);
                canvas.DrawRect(new SKRect(0, 0, upscaledW, upscaledH), paint);
                canvas.Restore();
            }
        }
    }
}
