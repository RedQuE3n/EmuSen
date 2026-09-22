using System;
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

        private readonly Task<SlangChain?> _build;
        private readonly Action<string>? _failed;
        private SKImage? _output;
        private bool _advanced, _disposed;

        public string PresetPath { get; }

        // Why the preset is not drawn, or null while it is being built or when it is.
        public string? Problem { get; private set; }

        public bool Built => _build.IsCompleted;

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
                return new SlangChain(device, SlangPreset.Load(PresetPath));
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

        // On the render thread, under the owner's lock: false draws nothing and the caller draws the picture plain.
        public bool Draw(SKCanvas canvas, byte[] rgba, int width, int height, int rowRepeat, bool newFrame, SKRect destination)
        {
            if (_disposed || !_build.IsCompletedSuccessfully || _build.Result is not { } chain) return false;
            float scale = Math.Max(Math.Abs(canvas.TotalMatrix.ScaleX), 0.01f);
            int pw = Math.Max(1, (int)Math.Round(destination.Width * scale)), ph = Math.Max(1, (int)Math.Round(destination.Height * scale));
            try
            {
                if (newFrame || !_advanced) { chain.Advance(rgba, width, height, rowRepeat); _advanced = true; }
                if (newFrame || _output is null || _output.Width != pw || _output.Height != ph)
                {
                    byte[] pixels = chain.Render(pw, ph);
                    _output?.Dispose();
                    _output = SKImage.FromPixelCopy(new SKImageInfo(pw, ph, SKColorType.Rgba8888, SKAlphaType.Opaque), pixels);
                }
            }
            catch (Exception e) when (e is InvalidOperationException or ArgumentException)
            {
                Fail($"{Path.GetFileName(PresetPath)} stopped: {e.Message}");
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
