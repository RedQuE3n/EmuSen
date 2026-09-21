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
    // Draws a raw RGBA8888 frame through Avalonia's own Skia backend - see EmuSen_Serenity.md §2.
    public sealed class GameFrameControl : Control
    {
        private byte[]? _rgba;
        private int _frameWidth;
        private int _frameHeight;

        // How many times each row of the frame is shown, stretched here rather than repeated in the frame - see EmuSen_Serenity.md §2.7.
        private int _rowRepeat = 1;

        // Each offer is a new version even of the same array, whose contents a core may have rewritten - see EmuSen_Serenity.md §2.6.
        private long _version;

        // The image made from the last version drawn, reused when the control is only redrawn; the render thread's, under the lock.
        private readonly object _cacheLock = new();
        private SKImage? _cachedImage;
        private long _cachedVersion = -1;

        private readonly Dictionary<ShaderEffect, SKRuntimeEffect> _effects = new();

        // One long-lived builder per effect, never rebuilt per frame - see EmuSen_Serenity.md §2.2.
        private readonly Dictionary<ShaderEffect, SKRuntimeShaderBuilder> _builders = new();

        // Not "Effect" - Avalonia's Visual already has an unrelated one - see EmuSen_Serenity.md §2.4.
        public ShaderEffect ActiveEffect { get; set; } = ShaderEffect.None;

        // What the render thread spent showing frames, summed until a frontend takes them - see EmuSen_Serenity.md §2.5.
        private long _presented, _copies, _copyTicks, _drawTicks, _shape;
        private int _gpu;

        public readonly record struct PresentationStatistics(long Frames, long Copies, double CopyMilliseconds, double DrawMilliseconds, bool Gpu, int Width, int Height);

        // Safe from any thread: the sums are exchanged for zero, so each frame is counted by exactly one take.
        public PresentationStatistics TakeStatistics()
        {
            double ms = 1000.0 / System.Diagnostics.Stopwatch.Frequency;
            long shape = System.Threading.Interlocked.Read(ref _shape);
            return new PresentationStatistics(
                System.Threading.Interlocked.Exchange(ref _presented, 0),
                System.Threading.Interlocked.Exchange(ref _copies, 0),
                System.Threading.Interlocked.Exchange(ref _copyTicks, 0) * ms,
                System.Threading.Interlocked.Exchange(ref _drawTicks, 0) * ms,
                System.Threading.Volatile.Read(ref _gpu) != 0,
                (int)(shape >> 32), (int)shape);
        }

        private void Presented(bool copied, long copyTicks, long drawTicks, bool gpu, int width, int height)
        {
            if (copied) System.Threading.Interlocked.Increment(ref _copies);
            System.Threading.Interlocked.Add(ref _copyTicks, copyTicks);
            System.Threading.Interlocked.Add(ref _drawTicks, drawTicks);
            System.Threading.Volatile.Write(ref _gpu, gpu ? 1 : 0);
            System.Threading.Interlocked.Exchange(ref _shape, (long)width << 32 | (uint)height);
            System.Threading.Interlocked.Increment(ref _presented);
        }

        // Stores the frame and asks for a repaint; drawing happens in Render() below.
        public void UpdateFrame(byte[] rgba, int width, int height, int rowRepeat = 1)
        {
            _rgba = rgba;
            _frameWidth = width;
            _frameHeight = height;
            _rowRepeat = Math.Max(1, rowRepeat);
            _version++;
            InvalidateVisual();
        }

        // Fits the frame, centered, inside the control's real bounds - see EmuSen_Serenity.md §2.1.
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

        // Compiles a built-in SkSL effect on first use and caches it - see EmuSen_Serenity.md §2.2.
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

        // Lives as long as the control; only Children/Uniforms rebind - see EmuSen_Serenity.md §2.2.
        private SKRuntimeShaderBuilder GetBuilder(ShaderEffect effect)
        {
            if (_builders.TryGetValue(effect, out SKRuntimeShaderBuilder? existing)) return existing;

            var builder = new SKRuntimeShaderBuilder(GetEffect(effect));
            _builders[effect] = builder;
            return builder;
        }

        // The cached image is native memory the size of a frame, so a control leaving the window gives it back.
        protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
        {
            base.OnDetachedFromVisualTree(e);
            lock (_cacheLock)
            {
                _cachedImage?.Dispose();
                _cachedImage = null;
                _cachedVersion = -1;
            }
        }

        public override void Render(DrawingContext context)
        {
            base.Render(context);
            if (_rgba == null || _frameWidth <= 0 || _frameHeight <= 0) return;

            context.Custom(new DrawOp(new Rect(Bounds.Size), this, _rgba, _frameWidth, _frameHeight, _rowRepeat, ActiveEffect, _version));
        }

        // An immutable snapshot per Render() call - see EmuSen_Serenity.md §2.3.
        private sealed class DrawOp : ICustomDrawOperation
        {
            private readonly GameFrameControl _owner;
            private readonly byte[] _rgba;
            private readonly int _width;
            private readonly int _height;
            private readonly ShaderEffect _effect;
            private readonly long _version;
            private readonly int _rowRepeat;

            public DrawOp(Rect bounds, GameFrameControl owner, byte[] rgba, int width, int height, int rowRepeat, ShaderEffect effect, long version)
            {
                _version = version;
                _rowRepeat = rowRepeat;
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

                lock (_owner._cacheLock)
                {
                    long started = System.Diagnostics.Stopwatch.GetTimestamp();
                    bool copy = _owner._cachedImage is null || _owner._cachedVersion != _version;
                    if (copy)
                    {
                        var sourceInfo = new SKImageInfo(_width, _height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
                        _owner._cachedImage?.Dispose();
                        _owner._cachedImage = SKImage.FromPixelCopy(sourceInfo, _rgba);
                        _owner._cachedVersion = _version;
                    }
                    long copied = System.Diagnostics.Stopwatch.GetTimestamp();

                    Draw(canvas, _owner._cachedImage!);

                    // Flushed here so the texture's upload, which Skia defers to a flush, is timed with the draw - see EmuSen_Serenity.md §2.5.
                    lease.GrContext?.Flush();
                    _owner.Presented(copy, copied - started, System.Diagnostics.Stopwatch.GetTimestamp() - copied, lease.GrContext is not null, _width, _height);
                }
            }

            private void Draw(SKCanvas canvas, SKImage sourceImage)
            {
                // GraphicsSettings.BilinearFiltering - see EmuSen_Settings_Reference.md §3.
                SKSamplingOptions sampling = GraphicsSettings.BilinearFiltering
                    ? new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None)
                    : new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None);

                var (x, y, w, h) = ComputeLetterboxRect(_width, _height * _rowRepeat, Bounds.Width, Bounds.Height);
                int upscaledW = Math.Max(1, (int)Math.Round(w));
                int upscaledH = Math.Max(1, (int)Math.Round(h));
                var destRect = new SKRect((float)x, (float)y, (float)x + upscaledW, (float)y + upscaledH);

                // No offscreen surface on either path - see EmuSen_Serenity.md §2.2.
                if (_effect == ShaderEffect.None)
                {
                    canvas.DrawImage(sourceImage, destRect, sampling);
                    return;
                }

                // A local matrix is what puts the shader's math in output-pixel space - see EmuSen_Serenity.md §3.
                float scaleX = upscaledW / (float)_width;
                float scaleY = upscaledH / (float)_height;

                SKRuntimeShaderBuilder builder = _owner.GetBuilder(_effect);
                builder.Children["image"] = sourceImage.ToShader(
                    SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, sampling, SKMatrix.CreateScale(scaleX, scaleY));
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
