using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Skia.Helpers;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Serenity;
using SkiaSharp;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A themed view's full redraw on a GPU Skia surface over a surfaceless EGL context, beside the CPU's, pixels compared - see EmuSen_BigPicture.md §14.
    public static unsafe class SceneGpuBench
    {
        // One timed configuration: which view, what size, and a change applied to the window before timing.
        public sealed record Variant(string Name, Action<Window>? Apply = null);

        public static IReadOnlyList<Variant> NoLevers { get; } = [new("as built")];

        private static readonly Type ImagePixels = typeof(EmuSen.LunaP.Controls.FittedImage).Assembly.GetType("EmuSen.LunaP.Media.ImagePixels")!;

        // P27's lever: every processed picture handed to Skia as an immutable bitmap of the same premultiplied pixels.
        public static Variant ImmutableBitmaps { get; } = new("immutable bitmaps", window =>
        {
            window.CaptureRenderedFrame()?.Dispose();
            var prepared = (System.Collections.IDictionary)ImagePixels.GetField("Prepared", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!.GetValue(null)!;
            foreach (object key in prepared.Keys.Cast<object>().ToList())
            {
                if (prepared[key] is not WriteableBitmap wb) continue;
                using ILockedFramebuffer fb = wb.Lock();
                prepared[key] = new Bitmap(fb.Format, AlphaFormat.Premul, fb.Address, fb.Size, fb.Dpi, fb.RowBytes);
            }
        });

        // Every variant the handheld bench times after the as-built one.
        public static IReadOnlyList<Variant> Levers => [ImmutableBitmaps];

        // Every case starts from no processed pictures, so a lever's bitmaps never reach the next case.
        private static void ForgetPictures() => ImagePixels.GetMethod("Forget", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!.Invoke(null, null);

        // Runs every size and view on the calling thread, which must be the headless session's UI thread; returns one summary line per case.
        public static List<string> Run(string theme, string media, IEnumerable<(int W, int H)> sizes, IEnumerable<string> views, int frames, string? device, Action<string> log,
            string? pngFolder = null, IReadOnlyList<Variant>? variants = null)
        {
            variants ??= NoLevers;
            var lines = new List<string>();
            using var gl = ShaderBench.GlContext.Create(device);
            log($"gl: {gl.Renderer}");
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(ShaderBench.GlContext.GetProc) ?? throw new InvalidOperationException("no GL interface");
            using GRContext context = GRContext.CreateGl(glInterface) ?? throw new InvalidOperationException("no GRContext");
            uint query = gl.NewQuery();
            SyntheticLibrary.WriteMedia(media);
            var references = new Dictionary<(string, int), byte[]>();

            foreach ((int w, int h) in sizes)
            {
                IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(theme, new ThemeChoices { ScreenWidth = w, ScreenHeight = h });
                var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul);
                foreach (string view in views)
                    foreach (Variant variant in variants)
                    {
                        ForgetPictures();
                        SceneData data = SyntheticLibrary.Data(systems, new Size(w, h), media, system: 1, game: 2);
                        SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View(view), data);
                        var window = new Window { Width = w, Height = h, Content = scene.Canvas, Background = Brushes.Black, SizeToContent = SizeToContent.Manual };
                        window.Show();
                        Dispatcher.UIThread.RunJobs();
                        variant.Apply?.Invoke(window);
                        Dispatcher.UIThread.RunJobs();

                        byte[] compositor = Compositor(window, out double compositorMs);
                        using SKSurface cpu = SKSurface.Create(info) ?? throw new InvalidOperationException("no raster surface");
                        using SKSurface gpu = SKSurface.Create(context, true, info) ?? throw new InvalidOperationException("no GPU surface");
                        using var nodraw = new SKNoDrawCanvas(w, h);

                        var first = Stopwatch.StartNew();
                        Draw(gpu.Canvas, window, w, h);
                        context.Flush(true, true);
                        double firstGpuMs = first.Elapsed.TotalMilliseconds;

                        Timing record = Time(frames, () => Draw(nodraw, window, w, h), null, null, 0);
                        Timing cpuTiming = Time(frames, () => { cpu.Canvas.Clear(SKColors.Black); Draw(cpu.Canvas, window, w, h); cpu.Canvas.Flush(); }, null, null, 0);
                        Timing gpuTiming = Time(frames, () => { gpu.Canvas.Clear(SKColors.Black); Draw(gpu.Canvas, window, w, h); }, context, gl, query);

                        byte[] cpuPixels = Read(cpu, info), gpuPixels = Read(gpu, info);
                        Difference immediate = Compare(cpuPixels, compositor), device2 = Compare(gpuPixels, compositor), raster = Compare(gpuPixels, cpuPixels);
                        references.TryAdd((view, w), gpuPixels);
                        Difference lever = Compare(gpuPixels, references[(view, w)]);
                        string name = $"{view} {w}x{h} [{variant.Name}]";
                        if (pngFolder is not null)
                        {
                            string stem = $"{view}-{w}x{h}-{variant.Name.Replace(' ', '-')}";
                            Png(System.IO.Path.Combine(pngFolder, stem + "-gpu.png"), gpuPixels, w, h);
                            Png(System.IO.Path.Combine(pngFolder, stem + "-cpu.png"), cpuPixels, w, h);
                            Png(System.IO.Path.Combine(pngFolder, stem + "-compositor.png"), compositor, w, h);
                            Png(System.IO.Path.Combine(pngFolder, stem + "-diff-gpu.png"), DiffImage(gpuPixels, compositor), w, h);
                            Png(System.IO.Path.Combine(pngFolder, stem + "-diff-cpu.png"), DiffImage(cpuPixels, compositor), w, h);
                        }

                        log($"{name}: first GPU frame {firstGpuMs:F1} ms; compositor (headless CPU, readback included) {compositorMs:F2} ms");
                        log($"  controls' Render into a no-draw canvas   {record}");
                        log($"  CPU raster, immediate                    {cpuTiming}   vs compositor: {immediate}");
                        log($"  GPU: record+flush on the CPU             {gpuTiming}");
                        log($"  GPU: GL time                             {gpuTiming.Gpu}");
                        log($"  GPU: wall to glFinish                    {gpuTiming.Wall}   vs compositor: {device2}");
                        log($"  GPU against CPU immediate: {raster}; against the first variant on the GPU: {lever}");
                        lines.Add(FormattableString.Invariant(
                            $"case={view}-{w}x{h} lever={variant.Name.Replace(' ', '_')} gl=\"{gl.Renderer}\" record={record.Cpu.Median:F2} cpu={cpuTiming.Cpu.Median:F2} gpu.cpu={gpuTiming.Cpu.Median:F2} gpu.gl={gpuTiming.Gpu.Median:F2} gpu.wall={gpuTiming.Wall.Median:F2} gpu.wall.p95={gpuTiming.Wall.P95:F2} first={firstGpuMs:F1} cpu.vs.compositor={immediate.Short} gpu.vs.compositor={device2.Short} gpu.vs.cpu={raster.Short} vs.first.variant={lever.Short}"));
                        window.Close();
                        context.PurgeResources();
                    }
            }

            return lines;
        }

        // One moving case: a view held moving by a script of presses, each frame's clock step, layout and GPU draw timed at 60 Hz of its own clock.
        public sealed record Motion(string Name, string View, Action<SceneView, TimeSpan> Script);

        // A carousel stepped again as each step ends, and a list held down: the two motions that redraw the most.
        public static IReadOnlyList<Motion> Motions { get; } =
        [
            new("carousel held", "system", (v, now) => { if (!v.IsMoving || now == TimeSpan.Zero) v.Step(1, now); }),
            new("list held", "gamelist", (v, now) => { if (now == TimeSpan.Zero) v.Press(1, now); }),
        ];

        public static List<string> RunMotions(string theme, string media, IEnumerable<(int W, int H)> sizes, int frames, string? device, Action<string> log, IReadOnlyList<Motion>? motions = null)
        {
            motions ??= Motions;
            var lines = new List<string>();
            using var gl = ShaderBench.GlContext.Create(device);
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(ShaderBench.GlContext.GetProc) ?? throw new InvalidOperationException("no GL interface");
            using GRContext context = GRContext.CreateGl(glInterface) ?? throw new InvalidOperationException("no GRContext");
            uint query = gl.NewQuery();
            SyntheticLibrary.WriteMedia(media);
            foreach ((int w, int h) in sizes)
            {
                IReadOnlyList<SceneSystem> systems = SyntheticLibrary.Load(theme, new ThemeChoices { ScreenWidth = w, ScreenHeight = h });
                var info = new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul);
                foreach (Motion motion in motions)
                {
                    ForgetPictures();
                    SceneData data = SyntheticLibrary.Data(systems, new Size(w, h), media, system: 1, game: 0);
                    var view = new SceneView(data, motion.View, TimeSpan.Zero);
                    var window = new Window { Width = w, Height = h, Content = view.Root, Background = Brushes.Black, SizeToContent = SizeToContent.Manual };
                    window.Show();
                    Dispatcher.UIThread.RunJobs();
                    using SKSurface gpu = SKSurface.Create(context, true, info) ?? throw new InvalidOperationException("no GPU surface");
                    var update = new List<double>();
                    var cpu = new List<double>();
                    var gpuMs = new List<double>();
                    var wall = new List<double>();
                    for (int i = 0; i < frames + 5; i++)
                    {
                        TimeSpan now = TimeSpan.FromTicks(i * TimeSpan.TicksPerSecond / 60);
                        long t0 = Stopwatch.GetTimestamp();
                        motion.Script(view, now);
                        view.Advance(now);
                        window.UpdateLayout();
                        long t1 = Stopwatch.GetTimestamp();
                        gl.BeginQuery(query);
                        gpu.Canvas.Clear(SKColors.Black);
                        Draw(gpu.Canvas, window, w, h);
                        context.Flush(true, false);
                        long t2 = Stopwatch.GetTimestamp();
                        gl.EndQuery();
                        gl.Finish();
                        long t3 = Stopwatch.GetTimestamp();
                        if (i < 5) continue;
                        update.Add(Ms(t1 - t0));
                        cpu.Add(Ms(t2 - t1));
                        wall.Add(Ms(t3 - t0));
                        gpuMs.Add(gl.QueryNanoseconds(query) / 1e6);
                    }

                    Stat u = Of(update), c = Of(cpu), g = Of(gpuMs), all = Of(wall);
                    log($"{motion.Name} {w}x{h}: clock step and layout {u}; GPU record+flush {c}; GL {g}; whole frame {all}, max {wall.Max():F2} ms");
                    lines.Add(FormattableString.Invariant($"case=motion-{motion.Name.Replace(' ', '-')}-{w}x{h} update={u.Median:F2}/{u.P95:F2} gpu.cpu={c.Median:F2} gpu.gl={g.Median:F2} frame={all.Median:F2}/{all.P95:F2} max={wall.Max():F2}"));
                    window.Close();
                    context.PurgeResources();
                }
            }

            return lines;
        }

        // A full redraw through the headless compositor: every visual invalidated, then the frame read back as RGBA.
        private static byte[] Compositor(Window window, out double ms)
        {
            window.CaptureRenderedFrame()?.Dispose();
            var times = new List<double>();
            byte[] rgba = Array.Empty<byte>();
            for (int i = 0; i < 8; i++)
            {
                foreach (Visual v in window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
                var t = Stopwatch.StartNew();
                using WriteableBitmap bitmap = window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
                times.Add(t.Elapsed.TotalMilliseconds);
                if (i < 7) continue;
                using ILockedFramebuffer fb = bitmap.Lock();
                rgba = new byte[fb.Size.Width * fb.Size.Height * 4];
                for (int y = 0; y < fb.Size.Height; y++) Marshal.Copy(fb.Address + y * fb.RowBytes, rgba, y * fb.Size.Width * 4, fb.Size.Width * 4);
                if (fb.Format == PixelFormat.Bgra8888)
                    for (int p = 0; p < rgba.Length; p += 4) (rgba[p], rgba[p + 2]) = (rgba[p + 2], rgba[p]);
            }

            times.Sort();
            ms = times[times.Count / 2];
            return rgba;
        }

        private static void Draw(SKCanvas canvas, Window window, int w, int h) =>
            DrawingContextHelper.RenderAsync(canvas, window, new Rect(0, 0, w, h), new Vector(96, 96)).GetAwaiter().GetResult();

        public readonly record struct Stat(double Median, double P95)
        {
            public override string ToString() => FormattableString.Invariant($"median {Median,6:F2} ms, p95 {P95,6:F2}");
        }

        public readonly record struct Timing(Stat Cpu, Stat Gpu, Stat Wall)
        {
            public override string ToString() => Cpu.ToString();
        }

        private static Stat Of(List<double> v)
        {
            if (v.Count == 0) return default;
            v.Sort();
            return new Stat(v[v.Count / 2], v[Math.Min(v.Count - 1, (int)(v.Count * 0.95))]);
        }

        // Frames after five warm-up ones; on the GPU, the CPU's share (record and flush), the GL timer's, and the wall time to glFinish.
        private static Timing Time(int frames, Action draw, GRContext? context, ShaderBench.GlContext? gl, uint query)
        {
            var cpu = new List<double>();
            var gpu = new List<double>();
            var wall = new List<double>();
            for (int i = 0; i < frames + 5; i++)
            {
                gl?.BeginQuery(query);
                long t0 = Stopwatch.GetTimestamp();
                draw();
                context?.Flush(true, false);
                long t1 = Stopwatch.GetTimestamp();
                gl?.EndQuery();
                gl?.Finish();
                long t2 = Stopwatch.GetTimestamp();
                if (i < 5) continue;
                cpu.Add(Ms(t1 - t0));
                if (gl is null) continue;
                wall.Add(Ms(t2 - t0));
                gpu.Add(gl.QueryNanoseconds(query) / 1e6);
            }

            return new Timing(Of(cpu), Of(gpu), Of(wall));
        }

        private static double Ms(long ticks) => ticks * 1000.0 / Stopwatch.Frequency;

        private static byte[] Read(SKSurface surface, SKImageInfo info)
        {
            var pixels = new byte[info.Width * info.Height * 4];
            fixed (byte* p = pixels) surface.ReadPixels(info, (nint)p, info.Width * 4, 0, 0);
            return pixels;
        }

        // How far two RGBA frames are apart: pixels differing at all, by more than 2 and by more than 8 levels, the largest and the mean difference.
        public readonly record struct Difference(int Any, int Over2, int Over8, int Max, double Mean, int Total)
        {
            public string Short => FormattableString.Invariant($"{100.0 * Any / Total:F2}%/{100.0 * Over2 / Total:F2}%/{100.0 * Over8 / Total:F3}%/max{Max}/mean{Mean:F3}");

            public override string ToString() => FormattableString.Invariant(
                $"differ {100.0 * Any / Total:F2}% of pixels, >2 levels {100.0 * Over2 / Total:F2}%, >8 levels {100.0 * Over8 / Total:F3}%, max {Max}, mean {Mean:F3}");
        }

        public static Difference Compare(byte[] a, byte[] b)
        {
            int any = 0, over2 = 0, over8 = 0, max = 0;
            long sum = 0;
            int total = Math.Min(a.Length, b.Length) / 4;
            for (int i = 0; i < total * 4; i += 4)
            {
                int d = Math.Max(Math.Max(Math.Abs(a[i] - b[i]), Math.Abs(a[i + 1] - b[i + 1])), Math.Abs(a[i + 2] - b[i + 2]));
                sum += d;
                if (d > 0) any++;
                if (d > 2) over2++;
                if (d > 8) over8++;
                max = Math.Max(max, d);
            }

            return new Difference(any, over2, over8, max, total == 0 ? 0 : (double)sum / total, Math.Max(1, total));
        }

        // The largest channel difference per pixel, times eight, as grey on black.
        public static byte[] DiffImage(byte[] a, byte[] b)
        {
            var d = new byte[a.Length];
            for (int i = 0; i < a.Length; i += 4)
            {
                int v = Math.Max(Math.Max(Math.Abs(a[i] - b[i]), Math.Abs(a[i + 1] - b[i + 1])), Math.Abs(a[i + 2] - b[i + 2]));
                d[i] = d[i + 1] = d[i + 2] = (byte)Math.Min(255, v * 8);
                d[i + 3] = 255;
            }

            return d;
        }

        public static void Png(string path, byte[] rgba, int w, int h)
        {
            System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
            fixed (byte* p = rgba)
            {
                using var image = SKImage.FromPixels(new SKImageInfo(w, h, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, w * 4);
                using SKData png = image.Encode(SKEncodedImageFormat.Png, 100);
                using var file = System.IO.File.Create(path);
                png.SaveTo(file);
            }
        }
    }
}
