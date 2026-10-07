using System;
using System.Collections.Generic;
using System.Linq;
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

        public ScreenFilter Filter { get; }

        // The console whose defaults the filter's parameters start from, or null for the parameters' own.
        public string? Console { get; }

        public int HistoryHeld => _history.Count;

        // How many times the passes were compiled, for a test that asks when a filter is rebuilt.
        public int Builds { get; private set; }

        public int PassCount { get { EnsureBuilt(); return _passes.Count; } }

        public FilterChain(ScreenFilter filter) : this(filter, null) { }

        public FilterChain(ScreenFilter filter, string? console)
        {
            Filter = filter;
            Console = console;
            SetParameters(null);
            if (filter.Build is null) EnsureBuilt();
        }

        private readonly Dictionary<string, float> _values = new(StringComparer.Ordinal);

        // Values by id over the filter's defaults; an id it does not declare is ignored, one left out is its default - see EmuSen_Serenity.md §3.7.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            foreach (Slang.SlangParameter parameter in Filter.Parameters ?? Array.Empty<Slang.SlangParameter>())
            {
                float value = values is not null && values.TryGetValue(parameter.Id, out float given) ? given : Filter.DefaultFor(parameter, Console);
                bool structural = Filter.Structural is { } ids && ids.Contains(parameter.Id);
                if (structural && _built && (!_values.TryGetValue(parameter.Id, out float held) || held != value)) Unbuild();
                _values[parameter.Id] = value;
            }
        }

        public float ValueOf(string id) => _values.TryGetValue(id, out float value) ? value : float.NaN;

        // The passes compiled, from the filter's own list or from its values; a built filter is compiled at its first draw - see EmuSen_Serenity.md §3.9.
        private void EnsureBuilt()
        {
            if (_built) return;
            IReadOnlyList<FilterPass> passes = Filter.Build?.Invoke(_values) ?? Filter.Passes;
            var effects = new SKRuntimeEffect[passes.Count];
            for (int i = 0; i < passes.Count; i++)
            {
                effects[i] = SKRuntimeEffect.CreateShader(passes[i].Sksl, out string errors);
                if (effects[i] is not null) continue;
                for (int k = 0; k < i; k++) effects[k].Dispose();
                throw new InvalidOperationException($"Pass {i} of the '{Filter.Name}' filter did not compile: {errors}");
            }
            _passes = passes;
            _effects = effects;
            _builders = effects.Select(effect => new SKRuntimeShaderBuilder(effect)).ToArray();
            _surfaces = new SKSurface?[passes.Count];
            _earlier = new SKSurface?[passes.Count];
            _built = true;
            Builds++;
        }

        private void Unbuild()
        {
            DisposeSurfaces();
            foreach (SKRuntimeShaderBuilder builder in _builders) builder.Dispose();
            foreach (SKRuntimeEffect effect in _effects) effect.Dispose();
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

        public void Draw(SKCanvas canvas, GRContext? context, SKImage original, int rowRepeat, SKRect destination)
        {
            EnsureBuilt();
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

                    using SKShader shader = Bind(i, pass, source, sourceIsOriginal, original, rowRepeat, made, named, feedback,
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
        }

        // The rectangle shown, in the canvas's units and in the device's pixels.
        private readonly record struct View(int Width, int Height, float PixelScale, float OriginX, float OriginY);

        // Every child and uniform a pass declares, and only those; a missing history frame is the frame itself.
        private SKShader Bind(int index, FilterPass pass, SKImage source, bool sourceIsOriginal, SKImage original, int rowRepeat, List<IDisposable> made,
            Dictionary<string, Output> named, SKImage? feedback, int sourceWidth, int sourceHeight, int originalWidth, int originalHeight, int width, int height, View view)
        {
            SKRuntimeEffect effect = _effects[index];
            SKRuntimeShaderBuilder builder = _builders[index];
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
            Unbuild();
            foreach (SKImage image in _history) image.Dispose();
            _history.Clear();
        }
    }
}
