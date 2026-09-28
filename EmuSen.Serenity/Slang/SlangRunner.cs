using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using System.Threading.Tasks;
using SkiaSharp;

namespace EmuSen.Serenity.Slang
{
    // A preset built off the render thread and drawn on it; until it is built, or when it cannot be, the picture is drawn plain - see EmuSen_Serenity.md §7.5.
    public sealed class SlangRunner : IDisposable
    {
        // One device for the process, made on first use and kept, since every preset the player tries would otherwise open another.
        private static readonly Lazy<(SlangVulkan? Device, string Report)> Shared = new(() =>
        {
            SlangVulkan? device = SlangVulkan.TryCreate(out string report);
            return (device, report);
        }, LazyThreadSafetyMode.ExecutionAndPublication);

        // One SPIR-V cache for the process, opened on the first build; one that cannot open leaves every build compiling - see EmuSen_Serenity.md §9.4.
        private static readonly Lazy<SpirvCache> SharedCache = new(() => new SpirvCache(CachePath ?? SpirvCache.DefaultPath), LazyThreadSafetyMode.ExecutionAndPublication);

        // Where the shared cache is opened, when not the default; a test sets it before any preset is built.
        internal static string? CachePath;

        public static SpirvCache Cache => SharedCache.Value;

        private readonly Task<SlangChain?> _build;
        private readonly Action<string>? _failed;
        private SKImage? _output;
        private bool _advanced, _disposed;
        private IReadOnlyDictionary<string, float>? _values;
        private int _valuesVersion, _appliedVersion = -1;

        public string PresetPath { get; }

        // Why the preset is not drawn, or null while it is being built or when it is.
        public string? Problem { get; private set; }

        public bool Built => _build.IsCompleted;

        // Built, drawable and not stopped: the frame control then makes no image of the source - see EmuSen_Serenity.md §9.3.
        internal bool Ready => !_disposed && _build.IsCompletedSuccessfully && _build.Result is not null;

        // The built chain, for a test that counts its readbacks.
        internal SlangChain? Chain => _build.IsCompletedSuccessfully ? _build.Result : null;

        // Failed is called on whichever thread found the problem, once; built is called when building ends either way.
        public SlangRunner(string presetPath, Action<string>? failed = null, Action? built = null)
        {
            PresetPath = presetPath;
            _failed = failed;
            _build = Task.Run(Build);
            if (built is not null) _build.ContinueWith(_ => built(), TaskScheduler.Default);
        }

        public static string DeviceReport => Shared.Value.Report;

        private SlangChain? Build()
        {
            try
            {
                var (device, report) = Shared.Value;
                if (device is null) { Fail(report); return null; }
                return new SlangChain(device, SlangPreset.Load(PresetPath), Cache);
            }
            catch (Exception e) when (e is IOException or InvalidDataException or SlangCompileException or InvalidOperationException or ArgumentException or UnauthorizedAccessException)
            {
                Fail($"{Path.GetFileName(PresetPath)}: {e.Message.Split('\n')[0]}");
                return null;
            }
        }

        private void Fail(string problem)
        {
            Problem = problem;
            _failed?.Invoke(problem);
        }

        // Under the owner's lock; the chain takes them at its next draw, which renders again even with no new frame - see EmuSen_Serenity.md §7.6.
        public void SetParameters(IReadOnlyDictionary<string, float>? values)
        {
            _values = values;
            _valuesVersion++;
        }

        // On the render thread, under the owner's lock: false draws nothing and the caller draws the picture plain.
        public bool Draw(SKCanvas canvas, byte[] rgba, int width, int height, bool newFrame, SKRect destination)
        {
            if (_disposed || !_build.IsCompletedSuccessfully || _build.Result is not { } chain) return false;
            float scale = Math.Max(Math.Abs(canvas.TotalMatrix.ScaleX), 0.01f);
            int pw = Math.Max(1, (int)Math.Round(destination.Width * scale)), ph = Math.Max(1, (int)Math.Round(destination.Height * scale));
            try
            {
                bool retuned = _appliedVersion != _valuesVersion;
                if (retuned) { chain.SetParameters(_values); _appliedVersion = _valuesVersion; }
                // The rows once, as the console sent them and as a libretro core hands them over; the destination alone carries the repeat - see EmuSen_Serenity.md §10.6.
                if (newFrame || !_advanced) { chain.Advance(rgba, width, height, 1); _advanced = true; }
                if (newFrame || retuned || _output is null || _output.Width != pw || _output.Height != ph)
                {
                    // Let go first, so the readback it holds can take the new picture - see EmuSen_Serenity.md §9.1.
                    _output?.Dispose();
                    _output = null;
                    _output = chain.RenderImage(pw, ph);
                    SlangProbe.Current?.Phase(SlangProbe.ImageMade);
                }
            }
            catch (Exception e) when (e is InvalidOperationException or ArgumentException)
            {
                Fail($"{Path.GetFileName(PresetPath)} stopped: {e.Message}");
                _output?.Dispose();
                _output = null;
                chain.Dispose();
                _disposed = true;
                return false;
            }
            canvas.DrawImage(_output, destination, new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None));
            return true;
        }

        // Under the owner's lock, so no draw is using the chain; one still building is freed when it finishes.
        public void Dispose()
        {
            if (_disposed) return;
            _disposed = true;
            _output?.Dispose();
            _output = null;
            _build.ContinueWith(t => { if (t.IsCompletedSuccessfully) t.Result?.Dispose(); }, TaskScheduler.Default);
        }
    }
}
