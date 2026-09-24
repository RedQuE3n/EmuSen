using System;
using System.Collections.Generic;
using System.Linq;
using SkiaSharp;

namespace EmuSen.Serenity.Shaders
{
    // Runs a filter's passes over one frame, keeping the frames its passes look back at - see EmuSen_Serenity.md §3.2.
    internal sealed class FilterChain : IDisposable
    {
        private static readonly SKSamplingOptions Nearest = new(SKFilterMode.Nearest, SKMipmapMode.None);
        private static readonly SKSamplingOptions Linear = new(SKFilterMode.Linear, SKMipmapMode.None);

        private readonly SKRuntimeEffect[] _effects;
        private readonly SKRuntimeShaderBuilder[] _builders;
        private readonly SKSurface?[] _surfaces;
        private readonly List<SKImage> _history = new();
        private GRContext? _context;
        private long _frames;

        public ScreenFilter Filter { get; }

        public int HistoryHeld => _history.Count;

        public FilterChain(ScreenFilter filter)
        {
            Filter = filter;
            _effects = filter.Passes.Select((pass, i) => SKRuntimeEffect.CreateShader(pass.Sksl, out string errors)
                ?? throw new InvalidOperationException($"Pass {i} of the '{filter.Name}' filter did not compile: {errors}")).ToArray();
            _builders = _effects.Select(effect => new SKRuntimeShaderBuilder(effect)).ToArray();
            _surfaces = new SKSurface?[_effects.Length];
            SetParameters(null);
        }

        private readonly Dictionary<string, float> _values = new(StringComparer.Ordinal);

        // Values by id over the filter's defaults; an id it does not declare is ignored, one left out is its default - see EmuSen_Serenity.md §3.7.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            foreach (Slang.SlangParameter parameter in Filter.Parameters ?? Array.Empty<Slang.SlangParameter>())
                _values[parameter.Id] = values is not null && values.TryGetValue(parameter.Id, out float value) ? value : parameter.Initial;
        }

        public float ValueOf(string id) => _values.TryGetValue(id, out float value) ? value : float.NaN;

        // A new frame replaced the last: that one joins the history, which now owns it, or is freed.
        public void Advance(SKImage? previous)
        {
            _frames++;
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

        public void Draw(SKCanvas canvas, GRContext? context, SKImage original, int rowRepeat, SKRect destination)
        {
            if (!ReferenceEquals(context, _context))
            {
                DisposeSurfaces();
                _context = context;
            }

            int originalWidth = original.Width, originalHeight = original.Height * rowRepeat;
            int viewWidth = Math.Max(1, (int)Math.Round(destination.Width)), viewHeight = Math.Max(1, (int)Math.Round(destination.Height));
            SKImage source = original;
            bool sourceIsOriginal = true;
            int sourceWidth = originalWidth, sourceHeight = originalHeight;

            for (int i = 0; i < _effects.Length; i++)
            {
                FilterPass pass = Filter.Passes[i];
                bool last = i == _effects.Length - 1;
                (int width, int height) = pass.Scale == PassScale.Source ? (originalWidth, originalHeight) : (viewWidth, viewHeight);

                var children = new List<SKShader>();
                using SKShader shader = Build(i, pass, source, sourceIsOriginal, original, rowRepeat, children,
                    sourceWidth, sourceHeight, originalWidth, originalHeight, width, height);
                using var paint = new SKPaint { Shader = shader };

                if (last && pass.Scale == PassScale.Viewport)
                {
                    canvas.Save();
                    canvas.Translate(destination.Left, destination.Top);
                    canvas.DrawRect(new SKRect(0, 0, width, height), paint);
                    canvas.Restore();
                }
                else
                {
                    SKSurface surface = Surface(i, width, height);
                    surface.Canvas.Clear(SKColors.Black);
                    surface.Canvas.DrawRect(new SKRect(0, 0, width, height), paint);
                    SKImage output = surface.Snapshot();
                    if (!sourceIsOriginal) source.Dispose();
                    (source, sourceIsOriginal, sourceWidth, sourceHeight) = (output, false, width, height);
                    if (last) canvas.DrawImage(source, new SKRect(0, 0, width, height), destination, Nearest);
                }

                foreach (SKShader child in children) child.Dispose();
            }

            if (!sourceIsOriginal) source.Dispose();
        }

        // Every child and uniform a pass declares, and only those; a missing history frame is the frame itself.
        private SKShader Build(int index, FilterPass pass, SKImage source, bool sourceIsOriginal, SKImage original, int rowRepeat, List<SKShader> children,
            int sourceWidth, int sourceHeight, int originalWidth, int originalHeight, int width, int height)
        {
            SKRuntimeEffect effect = _effects[index];
            SKRuntimeShaderBuilder builder = _builders[index];
            SKMatrix stretch = SKMatrix.CreateScale(1, rowRepeat);

            SKShader Child(SKImage image, bool linear, bool stretched)
            {
                SKShader child = image.ToShader(SKShaderTileMode.Clamp, SKShaderTileMode.Clamp, linear ? Linear : Nearest, stretched ? stretch : SKMatrix.Identity);
                children.Add(child);
                return child;
            }

            foreach (string name in effect.Children)
            {
                builder.Children[name] = name switch
                {
                    "source" => Child(source, pass.LinearSource, sourceIsOriginal),
                    "original" => Child(original, false, true),
                    _ when name.StartsWith("history", StringComparison.Ordinal) && int.TryParse(name["history".Length..], out int back) && back >= 1 =>
                        Child(back <= _history.Count ? _history[back - 1] : original, false, true),
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
                    case "frameCount": builder.Uniforms[name] = (float)(_frames % 65536); break;
                    case var id when _values.TryGetValue(id, out float value): builder.Uniforms[name] = value; break;
                    default: throw new InvalidOperationException($"The '{Filter.Name}' filter asks for a uniform called '{name}', which no pass provides.");
                }
            }

            return builder.Build();
        }

        private SKSurface Surface(int index, int width, int height)
        {
            SKSurface? surface = _surfaces[index];
            if (surface is not null && surface.Canvas.DeviceClipBounds.Width == width && surface.Canvas.DeviceClipBounds.Height == height) return surface;
            surface?.Dispose();
            var info = new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Premul);
            surface = (_context is not null ? SKSurface.Create(_context, false, info) : null) ?? SKSurface.Create(info);
            _surfaces[index] = surface;
            return surface;
        }

        private void DisposeSurfaces()
        {
            for (int i = 0; i < _surfaces.Length; i++)
            {
                _surfaces[i]?.Dispose();
                _surfaces[i] = null;
            }
        }

        public void Dispose()
        {
            DisposeSurfaces();
            foreach (SKImage image in _history) image.Dispose();
            _history.Clear();
            foreach (SKRuntimeShaderBuilder builder in _builders) builder.Dispose();
            foreach (SKRuntimeEffect effect in _effects) effect.Dispose();
        }
    }
}
