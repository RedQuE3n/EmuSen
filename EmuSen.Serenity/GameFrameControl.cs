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
        // One offered picture, its rows' repeat (§2.7) and who can still read its array - see EmuSen_Serenity.md §2.8.
        internal sealed class Offer(byte[] rgba, int width, int height, int rowRepeat, long version, Action<byte[]>? release)
        {
            public readonly byte[] Rgba = rgba;
            public readonly int Width = width, Height = height, RowRepeat = rowRepeat;
            public readonly long Version = version;
            public readonly Action<byte[]>? Release = release;
            public int Readers;
            public bool Superseded, Released;
        }

        // Each offer is a new version even of the same array, whose contents a core may have rewritten - see EmuSen_Serenity.md §2.6.
        private long _version;

        // The bookkeeping of §2.8, taken briefly by every thread; never held across a copy or a draw.
        private readonly object _offerLock = new();
        private Offer? _current;

        // The offer the cached image was made from, kept readable, and the newest version ever copied, which only moves forward - see EmuSen_Serenity.md §2.8.
        private Offer? _cached;
        private long _copiedVersion = -1;

        // Superseded offers a draw operation may still copy; bounded, and one pushed out is left to the collector unreleased.
        private readonly List<Offer> _held = new(HeldLimit);
        private const int HeldLimit = 8;

        // The image made from the last version drawn, reused when the control is only redrawn; the render thread's, under the lock.
        private readonly object _cacheLock = new();
        private SKImage? _cachedImage;

        private readonly Dictionary<ShaderEffect, SKRuntimeEffect> _effects = new();

        // One long-lived builder per effect, never rebuilt per frame - see EmuSen_Serenity.md §2.2.
        private readonly Dictionary<ShaderEffect, SKRuntimeShaderBuilder> _builders = new();

        // Not "Effect" - Avalonia's Visual already has an unrelated one - see EmuSen_Serenity.md §2.4.
        public ShaderEffect ActiveEffect { get; set; } = ShaderEffect.None;

        // A multi-pass filter, drawn instead of ActiveEffect when set; its chain is built on the render thread - see EmuSen_Serenity.md §3.2.
        private ScreenFilter? _activeFilter;
        private FilterChain? _chain;

        public ScreenFilter? ActiveFilter
        {
            get => _activeFilter;
            set
            {
                lock (_cacheLock)
                {
                    if (ReferenceEquals(_activeFilter, value)) return;
                    _activeFilter = value;
                    _chain?.Dispose();
                    _chain = null;
                }
                InvalidateVisual();
            }
        }

        // A RetroArch preset, drawn instead of both when it is built; until then, or if it cannot be, the picture is drawn plain - see EmuSen_Serenity.md §7.5.
        private Slang.SlangRunner? _slang;

        // Raised off the UI thread, once per preset that cannot be drawn, with the reason.
        public event Action<string>? SlangFailed;

        public string? ActiveSlangPreset
        {
            get { lock (_cacheLock) return _slang?.PresetPath; }
            set
            {
                lock (_cacheLock)
                {
                    if (string.Equals(_slang?.PresetPath, value, StringComparison.Ordinal)) return;
                    _slang?.Dispose();
                    _slang = value is null ? null : new Slang.SlangRunner(value, problem => SlangFailed?.Invoke(problem),
                        () => Avalonia.Threading.Dispatcher.UIThread.Post(InvalidateVisual));
                    _slang?.SetParameters(_shaderParameters);
                }
                InvalidateVisual();
            }
        }

        private IReadOnlyDictionary<string, float>? _shaderParameters;

        // The player's values for the filter or preset drawn, by parameter id; both take them at their next draw - see EmuSen_Serenity.md §7.6.
        public IReadOnlyDictionary<string, float>? ShaderParameters
        {
            get { lock (_cacheLock) return _shaderParameters; }
            set
            {
                lock (_cacheLock)
                {
                    _shaderParameters = value is null ? null : new Dictionary<string, float>(value, StringComparer.Ordinal);
                    _chain?.SetParameters(_shaderParameters);
                    _slang?.SetParameters(_shaderParameters);
                }
                InvalidateVisual();
            }
        }

        // What the running filter's chain holds for a parameter, NaN when there is no chain yet, for a test.
        internal float FilterParameter(string id) { lock (_cacheLock) return _chain?.ValueOf(id) ?? float.NaN; }

        // Whether the preset has finished building, for a test that waits on it.
        internal bool SlangBuilt { get { lock (_cacheLock) return _slang?.Built ?? true; } }

        // How many earlier frames the running filter holds, for a test that asks whether history is kept.
        internal int FilterHistoryHeld { get { lock (_cacheLock) return _chain?.HistoryHeld ?? 0; } }

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

        // Stores the frame and asks for a repaint; release, if given, is called once with the array when nothing here can read it again - see EmuSen_Serenity.md §2.8.
        public void UpdateFrame(byte[] rgba, int width, int height, int rowRepeat = 1, Action<byte[]>? release = null)
        {
            lock (_offerLock)
            {
                Offer? previous = _current;
                _current = new Offer(rgba, width, height, Math.Max(1, rowRepeat), ++_version, release);
                if (previous is not null)
                {
                    previous.Superseded = true;
                    if (!TryRelease(previous)) Hold(previous);
                }
            }
            InvalidateVisual();
        }

        // Under _offerLock: an offer is dead once superseded, not the cache's, and read by nothing that could still copy it - see EmuSen_Serenity.md §2.8.
        private bool TryRelease(Offer offer)
        {
            if (offer.Released || !offer.Superseded || ReferenceEquals(offer, _cached)) return false;
            if (offer.Readers > 0 && offer.Version > _copiedVersion) return false;
            offer.Released = true;
            if (offer.Release is { } release && !StillOffered(offer)) release(offer.Rgba);
            return true;
        }

        // The same array offered again, as a core that rewrites its own buffer does, is not given back while a later offer holds it.
        private bool StillOffered(Offer offer)
        {
            if (ReferenceEquals(_current?.Rgba, offer.Rgba) || ReferenceEquals(_cached?.Rgba, offer.Rgba)) return true;
            foreach (Offer held in _held)
            {
                if (held != offer && !held.Released && ReferenceEquals(held.Rgba, offer.Rgba)) return true;
            }
            return false;
        }

        private void Hold(Offer offer)
        {
            if (_held.Count == HeldLimit) _held.RemoveAt(0);
            _held.Add(offer);
        }

        // Under _offerLock: whatever the newer copy has made dead is given back.
        private void ReleaseHeld()
        {
            for (int i = _held.Count - 1; i >= 0; i--)
            {
                if (_held[i].Released || TryRelease(_held[i])) _held.RemoveAt(i);
            }
        }

        // A draw operation's reference, taken when it is made and put down when it is disposed or has finished reading.
        private Offer? TakeReader(Offer? offer)
        {
            lock (_offerLock)
            {
                offer ??= _current;
                if (offer is not null) offer.Readers++;
                return offer;
            }
        }

        private void PutReader(Offer offer)
        {
            lock (_offerLock)
            {
                offer.Readers--;
                if (offer.Superseded) ReleaseHeld();
            }
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

        // The cached image is native memory the size of a frame, so a control leaving the window gives it back; the offer it came from stays, to be copied again.
        protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
        {
            base.OnDetachedFromVisualTree(e);
            lock (_cacheLock)
            {
                _cachedImage?.Dispose();
                _cachedImage = null;
                _chain?.Dispose();
                _chain = null;
            }
        }

        public override void Render(DrawingContext context)
        {
            base.Render(context);
            if (CaptureDrawOp(Bounds.Size) is { } op) context.Custom(op);
        }

        // What Render hands the compositor, holding the newest offer until disposed; the tests hold one and draw it when they like.
        internal DrawOp? CaptureDrawOp(Size size)
        {
            Offer? offer = TakeReader(null);
            if (offer is null) return null;
            if (offer.Width > 0 && offer.Height > 0) return new DrawOp(new Rect(size), this, offer, ActiveEffect);
            PutReader(offer);
            return null;
        }

        // The render thread's side of §2.8: the newest version ever copied is what a later draw shows, whichever operation draws it.
        private Offer BeginRead(Offer mine, out bool fresh)
        {
            lock (_offerLock)
            {
                fresh = mine.Version > _copiedVersion;
                Offer source = fresh ? mine : _cached!;
                source.Readers++;
                return source;
            }
        }

        private void Copied(Offer offer)
        {
            lock (_offerLock)
            {
                _cached = offer;
                _copiedVersion = offer.Version;
                ReleaseHeld();
            }
        }

        // One Render() call's offer, drawn forward only; the control's cache is under its lock - see EmuSen_Serenity.md §2.3 and §2.8.
        internal sealed class DrawOp : ICustomDrawOperation
        {
            private readonly GameFrameControl _owner;
            private readonly Offer _offer;
            private readonly ShaderEffect _effect;
            private bool _disposed;

            internal DrawOp(Rect bounds, GameFrameControl owner, Offer offer, ShaderEffect effect)
            {
                Bounds = bounds;
                _owner = owner;
                _offer = offer;
                _effect = effect;
            }

            public Rect Bounds { get; }

            public bool HitTest(Point p) => Bounds.Contains(p);

            public bool Equals(ICustomDrawOperation? other) => false; // always re-render - this is a live video feed, never cacheable

            public void Dispose()
            {
                lock (_owner._offerLock)
                {
                    if (_disposed) return;
                    _disposed = true;
                }
                _owner.PutReader(_offer);
            }

            public void Render(ImmediateDrawingContext context)
            {
                var feature = context.TryGetFeature<ISkiaSharpApiLeaseFeature>();
                if (feature == null) return; // non-Skia backend - nothing to draw through

                using ISkiaSharpApiLease lease = feature.Lease();
                RenderTo(lease.SkCanvas, lease.GrContext);
            }

            internal void RenderTo(SKCanvas canvas, GRContext? grContext)
            {
                lock (_owner._cacheLock)
                {
                    long started = System.Diagnostics.Stopwatch.GetTimestamp();
                    Offer source;
                    bool fresh;
                    lock (_owner._offerLock)
                    {
                        // In one hold with the read taken, or a Dispose between the two could give the array back first.
                        if (_disposed) return;
                        source = _owner.BeginRead(_offer, out fresh);
                    }
                    try
                    {
                        if (_owner._activeFilter is { } filter && _owner._chain is null) { _owner._chain = new FilterChain(filter); _owner._chain.SetParameters(_owner._shaderParameters); }

                        // A preset that will draw reads the array itself, so no image of the source is made, and one made earlier is now stale - see EmuSen_Serenity.md §9.3.
                        bool presetOnly = _owner._slang is { Ready: true } && _owner._chain is null;
                        if (presetOnly && fresh) { _owner._cachedImage?.Dispose(); _owner._cachedImage = null; }
                        bool copy = !presetOnly && (fresh || _owner._cachedImage is null);
                        if (copy) CopySource(source);
                        if (fresh) _owner.Copied(source);
                        long copied = System.Diagnostics.Stopwatch.GetTimestamp();

                        if (_owner._slang is { } slang && DrawSlang(canvas, slang, presetOnly ? fresh : copy, source)) { }
                        else
                        {
                            if (_owner._cachedImage is null) { CopySource(source); copy = true; }
                            if (_owner._chain is { } chain) DrawFiltered(canvas, chain, grContext, _owner._cachedImage!, source);
                            else Draw(canvas, _owner._cachedImage!, source);
                        }

                        // Flushed here so the texture's upload, which Skia defers to a flush, is timed with the draw - see EmuSen_Serenity.md §2.5.
                        Slang.SlangProbe.Current?.Phase(Slang.SlangProbe.FlushBegin);
                        grContext?.Flush();
                        Slang.SlangProbe.Current?.Phase(Slang.SlangProbe.FlushEnd);
                        _owner.Presented(copy, copied - started, System.Diagnostics.Stopwatch.GetTimestamp() - copied, grContext is not null, source.Width, source.Height);
                    }
                    finally
                    {
                        _owner.PutReader(source);
                    }
                }
            }

            private void CopySource(Offer source)
            {
                var sourceInfo = new SKImageInfo(source.Width, source.Height, SKColorType.Rgba8888, SKAlphaType.Unpremul);
                SKImage? previous = _owner._cachedImage;
                _owner._cachedImage = SKImage.FromPixelCopy(sourceInfo, source.Rgba);

                // A filter that looks back keeps the frame just replaced; otherwise it is freed as before.
                if (_owner._chain is { } running) running.Advance(previous);
                else previous?.Dispose();
            }

            private SKRect Destination(Offer source)
            {
                var (x, y, w, h) = ComputeLetterboxRect(source.Width, source.Height * source.RowRepeat, Bounds.Width, Bounds.Height);
                return new SKRect((float)x, (float)y, (float)x + Math.Max(1, (int)Math.Round(w)), (float)y + Math.Max(1, (int)Math.Round(h)));
            }

            private bool DrawSlang(SKCanvas canvas, Slang.SlangRunner slang, bool newFrame, Offer source) =>
                slang.Draw(canvas, source.Rgba, source.Width, source.Height, newFrame, Destination(source));

            private void DrawFiltered(SKCanvas canvas, FilterChain chain, GRContext? context, SKImage sourceImage, Offer source) =>
                chain.Draw(canvas, context, sourceImage, source.RowRepeat, Destination(source));

            private void Draw(SKCanvas canvas, SKImage sourceImage, Offer source)
            {
                // GraphicsSettings.BilinearFiltering - see EmuSen_Settings_Reference.md §3.
                SKSamplingOptions sampling = GraphicsSettings.BilinearFiltering
                    ? new SKSamplingOptions(SKFilterMode.Linear, SKMipmapMode.None)
                    : new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None);

                SKRect destRect = Destination(source);
                float x = destRect.Left, y = destRect.Top;
                int upscaledW = (int)Math.Round(destRect.Width), upscaledH = (int)Math.Round(destRect.Height);

                // No offscreen surface on either path - see EmuSen_Serenity.md §2.2.
                if (_effect == ShaderEffect.None)
                {
                    canvas.DrawImage(sourceImage, destRect, sampling);
                    return;
                }

                // A local matrix is what puts the shader's math in output-pixel space - see EmuSen_Serenity.md §3.
                float scaleX = upscaledW / (float)source.Width;
                float scaleY = upscaledH / (float)source.Height;

                SKRuntimeShaderBuilder builder = _owner.GetBuilder(_effect);
                builder.Children["image"] = sourceImage.ToShader(
                    SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, sampling, SKMatrix.CreateScale(scaleX, scaleY));
                builder.Uniforms["outputSize"] = new[] { (float)upscaledW, (float)upscaledH };

                using SKShader shader = builder.Build();
                using var paint = new SKPaint { Shader = shader };
                canvas.Save();
                canvas.Translate(x, y);
                canvas.DrawRect(new SKRect(0, 0, upscaledW, upscaledH), paint);
                canvas.Restore();
            }
        }
    }
}
