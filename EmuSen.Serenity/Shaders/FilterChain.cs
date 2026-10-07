using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using SkiaSharp;

namespace EmuSen.Serenity.Shaders
{
    // Runs a filter's passes over one frame, keeping the frames and the pass outputs its passes look back at - see EmuSen_Serenity.md §3.2 and §3.9.
    internal sealed class FilterChain : IDisposable
    {
        private static readonly SKSamplingOptions Nearest = new(SKFilterMode.Nearest, SKMipmapMode.None);
        private static readonly SKSamplingOptions Linear = new(SKFilterMode.Linear, SKMipmapMode.None);

        private IReadOnlyList<FilterPass> _passes = Array.Empty<FilterPass>();
        private SKRuntimeEffect[] _effects = Array.Empty<SKRuntimeEffect>();
        private SKRuntimeShaderBuilder[] _builders = Array.Empty<SKRuntimeShaderBuilder>();
        private SKSurface?[] _surfaces = Array.Empty<SKSurface?>();
        private SKSurface?[] _earlier = Array.Empty<SKSurface?>();
        private bool _built, _newFrame;
        private readonly List<SKImage> _history = new();
        private GRContext? _context;
        private long _frames;

        // Passes compiled together, ready to be drawn with or thrown away whole.
        private sealed record Compiled(IReadOnlyList<FilterPass> Passes, SKRuntimeEffect[] Effects, SKRuntimeShaderBuilder[] Builders) : IDisposable
        {
            public void Dispose()
            {
                foreach (SKRuntimeShaderBuilder builder in Builders) builder.Dispose();
                foreach (SKRuntimeEffect effect in Effects) effect.Dispose();
            }
        }

        // A build under way on a pool thread, and which request it answers, so a later request discards an earlier result - see EmuSen_Serenity.md §3.10.
        private Task<Compiled>? _pending;
        private int _requested;
        private readonly bool _background;

        public ScreenFilter Filter { get; }

        // The console whose defaults the filter's parameters start from, or null for the parameters' own.
        public string? Console { get; }

        public int HistoryHeld => _history.Count;

        // How many times the passes were compiled, for a test that asks when a filter is rebuilt.
        public int Builds => _builds;
        private int _builds;

        public int PassCount { get { Wait(); EnsureBuilt(); return _passes.Count; } }

        // Whether there are passes to draw with: the ones in use, or a build that has finished.
        public bool Ready => _built;

        // Whether its build has ended either way, so a failure can be thrown where the passes would have been drawn.
        public bool Settled => _built || _pending is { IsFaulted: true };

        // A filter that needs a device, given none, is drawn plain unless this is set, for a test of the software path - see EmuSen_Serenity.md §3.10.
        public bool DrawInSoftware { get; set; }

        // Compiled at once, for a caller that wants a filter's errors now.
        public FilterChain(ScreenFilter filter) : this(filter, null) => EnsureBuilt();

        public FilterChain(ScreenFilter filter, string? console) : this(filter, console, null, background: false) { }

        // Background, its builds run on a pool thread and the passes in use are drawn until one finishes - see EmuSen_Serenity.md §3.10.
        public FilterChain(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? values, bool background)
        {
            Filter = filter;
            Console = console;
            _background = background;
            SetParameters(values);
            if (!_built && _pending is null) Request();
        }

        private readonly Dictionary<string, float> _values = new(StringComparer.Ordinal);

        // Values by id over the filter's defaults; an id it does not declare is ignored, one left out is its default - see EmuSen_Serenity.md §3.7.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            bool rebuild = false;
            foreach (Slang.SlangParameter parameter in Filter.Parameters ?? Array.Empty<Slang.SlangParameter>())
            {
                float value = values is not null && values.TryGetValue(parameter.Id, out float given) ? given : Filter.DefaultFor(parameter, Console);
                bool structural = Filter.Structural is { } ids && ids.Contains(parameter.Id);
                if (structural && (!_values.TryGetValue(parameter.Id, out float held) || held != value)) rebuild = true;
                _values[parameter.Id] = value;
            }

            // A console's values for ids that are not parameters are its constants: no player sets them, and passes read them as any other value.
            if (Filter.DefaultsFor(Console) is { } constants) foreach (var (id, value) in constants) _values.TryAdd(id, value);
            if (rebuild && (_built || _pending is not null)) Request();
        }

        public float ValueOf(string id) => _values.TryGetValue(id, out float value) ? value : float.NaN;

        // A build of the current values: now, or on a pool thread while the passes in use go on being drawn.
        private void Request()
        {
            int request = ++_requested;
            if (!_background)
            {
                Unbuild();
                Use(Compile(new Dictionary<string, float>(_values, StringComparer.Ordinal)));
                return;
            }
            var values = new Dictionary<string, float>(_values, StringComparer.Ordinal);
            Task<Compiled> build = Task.Run(() => Compile(values));
            _pending = build;
            _pendingRequest = request;
        }

        private int _pendingRequest;

        // Blocks until any build under way has finished, for a test or a caller that cannot draw without it.
        public void Wait()
        {
            try { _pending?.Wait(); } catch (AggregateException) { }
            TakeFinished();
            if (_staged is { } staged) { _staged = null; Unbuild(); Use(staged); }
        }

        private Compiled Compile(IReadOnlyDictionary<string, float> values)
        {
            IReadOnlyList<FilterPass> passes = Filter.Build?.Invoke(values) ?? Filter.Passes;
            var effects = new SKRuntimeEffect[passes.Count];
            for (int i = 0; i < passes.Count; i++)
            {
                effects[i] = SKRuntimeEffect.CreateShader(passes[i].Sksl, out string errors);
                if (effects[i] is not null) continue;
                for (int k = 0; k < i; k++) effects[k].Dispose();
                throw new InvalidOperationException($"Pass {i} of the '{Filter.Name}' filter did not compile: {errors}");
            }
            Interlocked.Increment(ref _builds);
            return new Compiled(passes, effects, effects.Select(effect => new SKRuntimeShaderBuilder(effect)).ToArray());
        }

        // The finished build of the latest request takes the place of the passes in use; a failed one is thrown here, where a draw would have.
        private void TakeFinished()
        {
            if (_pending is not { IsCompleted: true } done) return;
            _pending = null;
            if (done.IsFaulted) throw done.Exception!.GetBaseException();
            if (_pendingRequest != _requested) { done.Result.Dispose(); return; }
            _staged?.Dispose();
            (_staged, _warmed) = (done.Result, 0);
        }

        // A finished build waiting to be warmed: its passes are drawn once each at one pixel, a pass a draw, so the driver compiles one program a frame.
        private Compiled? _staged;
        private int _warmed;
        private readonly Dictionary<SKColorType, SKImage> _blanks = new();

        private SKImage Blank(SKColorType type)
        {
            if (_blanks.TryGetValue(type, out SKImage? image)) return image;
            using SKSurface surface = SKSurface.Create(new SKImageInfo(1, 1, type, SKAlphaType.Premul));
            surface.Canvas.Clear(SKColors.Black);
            return _blanks[type] = surface.Snapshot();
        }

        // Warms the staged build's next pass on the device, and puts the build in use once every pass has been drawn - see EmuSen_Serenity.md §3.10.
        private void Warm(GRContext? context)
        {
            if (_staged is not { } staged) return;
            if (context is not null && _warmed < staged.Passes.Count)
            {
                int i = _warmed++;
                FilterPass pass = staged.Passes[i];
                SKColorType Of(FilterPass p) => p.Float ? SKColorType.RgbaF16 : SKColorType.Rgba8888;
                var named = new Dictionary<string, Output>(StringComparer.Ordinal);
                for (int k = 0; k < i; k++) if (staged.Passes[k].Name is { } name) named[name] = new Output(Blank(Of(staged.Passes[k])), 1, 1);
                SKImage original = Blank(SKColorType.Rgba8888);
                SKImage source = i == 0 ? original : Blank(Of(staged.Passes[i - 1]));
                var made = new List<IDisposable>();
                try
                {
                    using SKShader shader = Bind(staged.Effects[i], staged.Builders[i], pass, source, i == 0, original, 1, made, named, pass.Feedback ? Blank(Of(pass)) : null,
                        1, 1, 1, 1, 1, 1, new View(1, 1, 1, 0, 0));
                    using var paint = new SKPaint { Shader = shader, BlendMode = pass.Float ? SKBlendMode.Src : SKBlendMode.SrcOver };
                    using SKSurface? surface = SKSurface.Create(context, false, new SKImageInfo(1, 1, Of(pass), SKAlphaType.Premul));
                    surface?.Canvas.DrawRect(new SKRect(0, 0, 1, 1), paint);
                    surface?.Flush();
                }
                finally
                {
                    foreach (IDisposable item in made) item.Dispose();
                }
                if (_warmed < staged.Passes.Count) return;
            }
            _staged = null;
            Unbuild();
            Use(staged);
        }

        private void EnsureBuilt()
        {
            if (_built) return;
            TakeFinished();
            if (!_built && !_background) Use(Compile(new Dictionary<string, float>(_values, StringComparer.Ordinal)));
        }

        private void Use(Compiled compiled)
        {
            _compiled = compiled;
            _passes = compiled.Passes;
            _effects = compiled.Effects;
            _builders = compiled.Builders;
            _surfaces = new SKSurface?[_passes.Count];
            _earlier = new SKSurface?[_passes.Count];
            _built = true;
        }

        private Compiled? _compiled;

        private void Unbuild()
        {
            DisposeSurfaces();
            _compiled?.Dispose();
            _compiled = null;
            (_passes, _effects, _builders, _built) = (Array.Empty<FilterPass>(), Array.Empty<SKRuntimeEffect>(), Array.Empty<SKRuntimeShaderBuilder>(), false);
        }

        // A new frame replaced the last: that one joins the history, which now owns it, or is freed.
        public void Advance(SKImage? previous)
        {
            _frames++;
            _newFrame = true;
            if (previous is null) return;
            if (Filter.History == 0)
            {
                previous.Dispose();
                return;
            }
            _history.Insert(0, previous);
            while (_history.Count > Filter.History)
            {
                _history[^1].Dispose();
                _history.RemoveAt(_history.Count - 1);
            }
        }

        private readonly record struct Output(SKImage Image, int Width, int Height);

        // A step of a chain not yet drawn with: its finished build taken, and one pass warmed on the device, so that it is ready when it takes over.
        public void Prepare(GRContext? context)
        {
            TakeFinished();
            Warm(context is null && Filter.RequiresDevice && !DrawInSoftware ? null : context);
        }

        // Draws the filtered picture and says so; false, nothing was drawn, and the caller draws the picture plain - see EmuSen_Serenity.md §3.10.
        public bool Draw(SKCanvas canvas, GRContext? context, SKImage original, int rowRepeat, SKRect destination)
        {
            if (context is null && Filter.RequiresDevice && !DrawInSoftware) return false;
            TakeFinished();
            Warm(context);
            EnsureBuilt();
            if (!_built) return false;
            if (!ReferenceEquals(context, _context))
            {
                DisposeSurfaces();
                _context = context;
            }

            // A feedback pass's two surfaces change places once per game frame, so a redraw reads the same earlier output.
            if (_newFrame)
            {
                for (int i = 0; i < _passes.Count; i++) if (_passes[i].Feedback) (_surfaces[i], _earlier[i]) = (_earlier[i], _surfaces[i]);
                _newFrame = false;
            }

            if (Filter.RowsOnce) rowRepeat = 1;
            int originalWidth = original.Width, originalHeight = original.Height * rowRepeat;
            int viewWidth = Math.Max(1, (int)Math.Round(destination.Width)), viewHeight = Math.Max(1, (int)Math.Round(destination.Height));
            SKMatrix total = canvas.TotalMatrix;
            var view = new View(viewWidth, viewHeight, total.ScaleX, total.TransX + destination.Left * total.ScaleX, total.TransY + destination.Top * total.ScaleY);
            SKImage source = original;
            bool sourceIsOriginal = true;
            int sourceWidth = originalWidth, sourceHeight = originalHeight;
            var named = new Dictionary<string, Output>(StringComparer.Ordinal);
            var made = new List<IDisposable>();

            try
            {
                for (int i = 0; i < _effects.Length; i++)
                {
                    FilterPass pass = _passes[i];
                    bool last = i == _effects.Length - 1;
                    (int width, int height) = pass.Scale == PassScale.Source ? (originalWidth, originalHeight) : (viewWidth, viewHeight);
                    if (pass.Width is { } w) width = w.Of(originalWidth, viewWidth);
                    if (pass.Height is { } h) height = h.Of(originalHeight, viewHeight);
                    bool direct = last && width == viewWidth && height == viewHeight && pass.Scale == PassScale.Viewport && !pass.Float && !pass.Feedback;

                    SKImage? feedback = null;
                    if (pass.Feedback)
                    {
                        feedback = Surface(_earlier, i, width, height, pass.Float).Snapshot();
                        made.Add(feedback);
                    }

                    using SKShader shader = Bind(_effects[i], _builders[i], pass, source, sourceIsOriginal, original, rowRepeat, made, named, feedback,
                        sourceWidth, sourceHeight, originalWidth, originalHeight, width, height, view);
                    using var paint = new SKPaint { Shader = shader };

                    if (direct)
                    {
                        canvas.Save();
                        canvas.Translate(destination.Left, destination.Top);
                        canvas.DrawRect(new SKRect(0, 0, width, height), paint);
                        canvas.Restore();
                        continue;
                    }

                    SKSurface surface = Surface(_surfaces, i, width, height, pass.Float);
                    if (pass.Float) paint.BlendMode = SKBlendMode.Src;
                    else surface.Canvas.Clear(SKColors.Black);
                    surface.Canvas.DrawRect(new SKRect(0, 0, width, height), paint);
                    SKImage output = surface.Snapshot();
                    made.Add(output);
                    (source, sourceIsOriginal, sourceWidth, sourceHeight) = (output, false, width, height);
                    if (pass.Name is { } name) named[name] = new Output(output, width, height);
                    if (last) canvas.DrawImage(source, new SKRect(0, 0, width, height), destination, Nearest);
                }
            }
            finally
            {
                foreach (IDisposable item in made) item.Dispose();
            }
            return true;
        }

        // The rectangle shown, in the canvas's units and in the device's pixels.
        private readonly record struct View(int Width, int Height, float PixelScale, float OriginX, float OriginY);

        // Every child and uniform a pass declares, and only those; a missing history frame is the frame itself.
        private SKShader Bind(SKRuntimeEffect effect, SKRuntimeShaderBuilder builder, FilterPass pass, SKImage source, bool sourceIsOriginal, SKImage original, int rowRepeat, List<IDisposable> made,
            Dictionary<string, Output> named, SKImage? feedback, int sourceWidth, int sourceHeight, int originalWidth, int originalHeight, int width, int height, View view)
        {
            SKMatrix stretch = SKMatrix.CreateScale(1, rowRepeat);

            SKShader Child(SKImage image, bool linear, bool stretched)
            {
                SKShader child = image.ToShader(SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, linear ? Linear : Nearest, stretched ? stretch : SKMatrix.Identity);
                made.Add(child);
                return child;
            }

            bool IsLinear(string name) => pass.Linear is { } names && names.Contains(name);

            foreach (string name in effect.Children)
            {
                builder.Children[name] = name switch
                {
                    "source" => Child(source, pass.LinearSource, sourceIsOriginal),
                    "original" => Child(original, IsLinear(name), true),
                    "feedback" when feedback is not null => Child(feedback, IsLinear(name), false),
                    _ when name.StartsWith("history", StringComparison.Ordinal) && int.TryParse(name["history".Length..], out int back) && back >= 1 =>
                        Child(back <= _history.Count ? _history[back - 1] : original, false, true),
                    _ when named.TryGetValue(name, out Output earlier) => Child(earlier.Image, IsLinear(name), false),
                    _ => throw new InvalidOperationException($"The '{Filter.Name}' filter asks for a child called '{name}', which no pass provides."),
                };
            }

            foreach (string name in effect.Uniforms)
            {
                switch (name)
                {
                    case "inputSize": builder.Uniforms[name] = new[] { (float)sourceWidth, sourceHeight }; break;
                    case "originalSize": builder.Uniforms[name] = new[] { (float)originalWidth, originalHeight }; break;
                    case "outputSize": builder.Uniforms[name] = new[] { (float)width, height }; break;
                    case "viewportSize": builder.Uniforms[name] = new[] { (float)view.Width, view.Height }; break;
                    case "pixelScale": builder.Uniforms[name] = view.PixelScale; break;
                    case "pixelOrigin": builder.Uniforms[name] = new[] { view.OriginX, view.OriginY }; break;
                    case "frameCount": builder.Uniforms[name] = (float)(_frames % 65536); break;
                    case var id when _values.TryGetValue(id, out float value): builder.Uniforms[name] = value; break;
                    case var size when size.EndsWith("Size", StringComparison.Ordinal) && named.TryGetValue(size[..^4], out Output earlier):
                        builder.Uniforms[name] = new[] { (float)earlier.Width, earlier.Height };
                        break;
                    default: throw new InvalidOperationException($"The '{Filter.Name}' filter asks for a uniform called '{name}', which no pass provides.");
                }
            }

            return builder.Build();
        }

        // Whether every surface asked for so far was made on the device, for a bench that would otherwise time an upload.
        public bool SurfacesOnDevice { get; private set; } = true;

        private SKSurface Surface(SKSurface?[] set, int index, int width, int height, bool floating)
        {
            SKSurface? surface = set[index];
            if (surface is not null && surface.Canvas.DeviceClipBounds.Width == width && surface.Canvas.DeviceClipBounds.Height == height) return surface;
            surface?.Dispose();
            var info = new SKImageInfo(width, height, floating ? SKColorType.RgbaF16 : SKColorType.Rgba8888, SKAlphaType.Premul);
            surface = _context is not null ? SKSurface.Create(_context, false, info) : null;
            if (surface is null)
            {
                if (_context is not null) SurfacesOnDevice = false;
                surface = SKSurface.Create(info);
            }
            surface.Canvas.Clear(floating ? SKColors.Transparent : SKColors.Black);
            set[index] = surface;
            return surface;
        }

        private void DisposeSurfaces()
        {
            foreach (SKSurface?[] set in new[] { _surfaces, _earlier })
            {
                for (int i = 0; i < set.Length; i++)
                {
                    set[i]?.Dispose();
                    set[i] = null;
                }
            }
        }

        public void Dispose()
        {
            _requested++;
            _pending?.ContinueWith(build => { if (build.IsCompletedSuccessfully) build.Result.Dispose(); }, TaskScheduler.Default);
            _pending = null;
            _staged?.Dispose();
            _staged = null;
            foreach (SKImage image in _blanks.Values) image.Dispose();
            _blanks.Clear();
            Unbuild();
            foreach (SKImage image in _history) image.Dispose();
            _history.Clear();
        }
    }
}
