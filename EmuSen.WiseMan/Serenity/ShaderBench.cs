using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using Avalonia;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using Silk.NET.Vulkan;
using SkiaSharp;

namespace EmuSen.WiseMan.Serenity
{
    // What one shaded frame costs on the render thread, stage by stage, on a real GL context and the slang device - see EmuSen_Serenity.md §8.1.
    public static unsafe class ShaderBench
    {
        public sealed record Case(string Kind, string Shader, int SourceWidth, int SourceHeight, int RowRepeat, int WindowWidth, int WindowHeight, int Frames, int Warmup, double Pace, string Cache = "", int Builders = 0, string Console = "", string Set = "", string Raw = "", string Path = "readback", string Consumer = "new");

        // A new frame every this many draws; the draws between redraw the same frame, as a paused or repeating game gives.
        public static int Every = Math.Max(1, int.Parse(Environment.GetEnvironmentVariable("EMUSEN_BENCH_EVERY") ?? "1"));

        public static Case Parse(IEnumerable<string> args)
        {
            var d = args.Select(a => a.Split('=', 2)).Where(p => p.Length == 2).ToDictionary(p => p[0], p => p[1]);
            static (int, int) Size(string s) { var p = s.Split('x'); return (int.Parse(p[0]), int.Parse(p[1])); }
            var (sw, sh) = Size(d.GetValueOrDefault("src", "256x224"));
            var (ww, wh) = Size(d.GetValueOrDefault("window", "1920x1080"));
            return new Case(d.GetValueOrDefault("kind", "none"), d.GetValueOrDefault("shader", ""), sw, sh, int.Parse(d.GetValueOrDefault("repeat", "1")),
                ww, wh, int.Parse(d.GetValueOrDefault("frames", "600")), int.Parse(d.GetValueOrDefault("warmup", "60")),
                double.Parse(d.GetValueOrDefault("pace", "60"), CultureInfo.InvariantCulture),
                d.GetValueOrDefault("cache", ""), int.Parse(d.GetValueOrDefault("builders", "0")), d.GetValueOrDefault("console", ""), d.GetValueOrDefault("set", ""),
                d.GetValueOrDefault("raw", ""), d.GetValueOrDefault("path", "readback"), d.GetValueOrDefault("consumer", "new"));
        }

        // A preset's build, whole and in parts, after a small preset has warmed the compiler and the device.
        public static string Load(Case c, Action<string> log)
        {
            using SlangVulkan gpu = SlangVulkan.TryCreate(out string report) ?? throw new InvalidOperationException(report);
            log($"device: {report}");
            string stock = Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(c.Shader))!, "stock.slang");
            if (!File.Exists(stock)) stock = Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(Path.GetDirectoryName(Path.GetDirectoryName(c.Shader))))!, "stock.slang");
            if (File.Exists(stock)) { var warm = SlangSource.Load(stock); SlangCompiler.Compile(warm.Vertex, SlangStage.Vertex, stock); }

            // cache= none (the default), a database's path, or fresh:<directory> for a new empty one; opened before the clock starts. builders= passes compiled at once (0: the default).
            string? fresh = c.Cache.StartsWith("fresh:", StringComparison.Ordinal) ? Path.Combine(c.Cache["fresh:".Length..], $"{Guid.NewGuid():N}.db") : null;
            using SpirvCache? cache = c.Cache is "" or "none" ? null : new SpirvCache(fresh ?? c.Cache);
            int builders = c.Builders > 0 ? c.Builders : SlangChain.DefaultBuilders;
            var clock = Stopwatch.StartNew();
            SlangPreset preset = SlangPreset.Load(c.Shader);
            double presetMs = clock.Elapsed.TotalMilliseconds;
            clock.Restart();
            using (var chain = new SlangChain(gpu, preset, cache, builders)) { }
            double chainMs = clock.Elapsed.TotalMilliseconds;
            string cached = cache is null ? "" : FormattableString.Invariant($" cache.hits={cache.Hits} cache.misses={cache.Misses} cache.stored={cache.Stored} cache.problem={(cache.Problem is null ? "none" : "yes")}");

            clock.Restart();
            var sources = preset.Passes.Select(p => SlangSource.Load(p.ShaderPath)).ToArray();
            double sourceMs = clock.Elapsed.TotalMilliseconds;
            clock.Restart();
            for (int i = 0; i < sources.Length; i++)
            {
                SlangCompiler.Compile(sources[i].Vertex, SlangStage.Vertex, preset.Passes[i].ShaderPath);
                SlangCompiler.Compile(sources[i].Fragment, SlangStage.Fragment, preset.Passes[i].ShaderPath);
            }
            double compileMs = clock.Elapsed.TotalMilliseconds;
            clock.Restart();
            foreach (SlangTextureSpec lut in preset.Textures) using (SKBitmap.Decode(lut.Path)) { }
            double lutMs = clock.Elapsed.TotalMilliseconds;
            if (fresh is not null)
            {
                cache!.Dispose();
                foreach (string file in new[] { fresh, fresh + "-wal", fresh + "-shm" }) File.Delete(file);
            }
            string line = FormattableString.Invariant($"kind=load shader={Path.GetFileNameWithoutExtension(c.Shader)} passes={preset.Passes.Count} luts={preset.Textures.Count} load.chain={chainMs:F0} load.preset={presetMs:F1} load.source={sourceMs:F0} load.compile={compileMs:F0} load.lut={lutMs:F0} load.rest={chainMs - sourceMs - compileMs - lutMs:F0} builders={builders}") + cached;
            log(line);
            return line;
        }

        // One case: returns a line of name=value medians, and writes the per-stage table to the log.
        public static string Run(Case c, Action<string> log)
        {
            if (c.Kind == "load") return Load(c, log);
            if (c.Path == "present") return DevicePath.Run(c, log);
            using var gl = GlContext.Create(Environment.GetEnvironmentVariable("EMUSEN_BENCH_GL_DEVICE"));
            log($"gl: {gl.Renderer}");
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => GlContext.GetProc(name)) ?? throw new InvalidOperationException("no GL interface");
            using GRContext context = GRContext.CreateGl(glInterface) ?? throw new InvalidOperationException("no GRContext");
            using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(c.WindowWidth, c.WindowHeight, SKColorType.Rgba8888, SKAlphaType.Premul))
                ?? throw new InvalidOperationException("no GPU surface");

            // A prototype build's interop lever needs this context's procedures; production has no such type.
            typeof(SlangChain).Assembly.GetType("EmuSen.Serenity.Slang.SlangProto")?.GetField("GlProc")?.SetValue(null, new Func<string, nint>(GlContext.GetProc));
            var control = new GameFrameControl();
            switch (c.Kind)
            {
                case "slang": control.ActiveSlangPreset = c.Shader; break;
                case "filter":
                    // console= the console whose defaults apply; set= id:value pairs joined by commas, the player's values.
                    if (c.Console != "") control.FilterConsole = c.Console;
                    if (c.Set != "") control.ShaderParameters = c.Set.Split(',').Select(p => p.Split(':')).ToDictionary(p => p[0], p => float.Parse(p[1], CultureInfo.InvariantCulture));
                    control.ActiveFilter = c.Shader == CrtFilter.Name ? CrtFilter.Filter : ScreenFilters.Find(c.Shader.Replace('_', ' ')).Filter ?? throw new ArgumentException($"no filter {c.Shader}");
                    break;
                case "effect": control.ActiveEffect = ScreenFilters.Find(c.Shader.Replace('_', ' ')).Effect; break;
                case "none": break;
                default: throw new ArgumentException(c.Kind);
            }

            control.WaitForFilter();
            double? shape = control.ActiveFilter?.Aspect;
            var frames = Enumerable.Range(0, 8).Select(k => Pattern(c.SourceWidth, c.SourceHeight, k)).ToArray();
            // raw=nes hands each frame a nes.ppu picture beside it, as MoonRT does, so the CRT draws the PPU's own signal.
            RawPicture?[] raws = Enumerable.Range(0, 8).Select(k => c.Raw == "nes" ? NesRaw(c.SourceWidth, c.SourceHeight, k) : (RawPicture?)null).ToArray();
            var probe = new Probe();
            var size = new Size(c.WindowWidth, c.WindowHeight);

            if (c.Kind == "slang")
            {
                var build = Stopwatch.StartNew();
                control.UpdateFrame(frames[0], c.SourceWidth, c.SourceHeight, c.RowRepeat);
                while (!control.SlangBuilt) System.Threading.Thread.Sleep(5);
                log($"built in {build.Elapsed.TotalMilliseconds:F0} ms");
                probe.Attach(RunnerDevice() ?? throw new InvalidOperationException("the preset did not build: no device"));
            }
            // A built-in filter or simple effect is drawn by a chain read back as well, so its stages are heard the same way.
            else if (c.Kind is "filter" or "effect" && RunnerDevice() is { } chainDevice) probe.Attach(chainDevice);
            SlangProbe.Current = probe;
            uint glQuery = gl.NewQuery();

            var rows = new List<Dictionary<string, double>>();
            long allocated = 0; int gen0 = 0, gen1 = 0, gen2 = 0; TimeSpan pause = TimeSpan.Zero;
            var clock = Stopwatch.StartNew();
            double next = 0;
            for (int i = 0; i < c.Warmup + c.Frames; i++)
            {
                if (c.Pace > 0)
                {
                    next += 1000.0 / c.Pace;
                    double wait = next - clock.Elapsed.TotalMilliseconds;
                    if (wait > 1.5) System.Threading.Thread.Sleep((int)(wait - 1));
                    while (clock.Elapsed.TotalMilliseconds < next) { }
                }
                bool measured = i >= c.Warmup;
                if (i == c.Warmup) { allocated = GC.GetTotalAllocatedBytes(true); gen0 = GC.CollectionCount(0); gen1 = GC.CollectionCount(1); gen2 = GC.CollectionCount(2); pause = GC.GetTotalPauseDuration(); }
                if (i % Every == 0) control.UpdateFrame(frames[i / Every % frames.Length], c.SourceWidth, c.SourceHeight, c.RowRepeat, raw: raws[i / Every % frames.Length]);
                using var op = control.CaptureDrawOp(size) ?? throw new InvalidOperationException("no draw operation");
                probe.Begin();
                gl.BeginQuery(glQuery);
                long a0 = GC.GetAllocatedBytesForCurrentThread();
                long t0 = Stopwatch.GetTimestamp();
                op.RenderTo(target.Canvas, context);
                long t1 = Stopwatch.GetTimestamp();
                long a1 = GC.GetAllocatedBytesForCurrentThread();
                gl.EndQuery();
                context.ResetContext();
                gl.Finish();
                long t2 = Stopwatch.GetTimestamp();
                double glGpu = gl.QueryNanoseconds(glQuery) / 1e6;
                context.ResetContext();
                if (!measured || (Every > 1 && i % Every == 0)) continue;
                var row = probe.Row(t0, t1);
                row["gl.finish.wait"] = Ms(t2 - t1);
                row["render.alloc.bytes"] = a1 - a0;
                row["gl.gpu"] = glGpu;
                rows.Add(row);
            }
            long bytes = GC.GetTotalAllocatedBytes(true) - allocated;
            int g0 = GC.CollectionCount(0) - gen0, g1 = GC.CollectionCount(1) - gen1, g2 = GC.CollectionCount(2) - gen2;
            double pauseMs = (GC.GetTotalPauseDuration() - pause).TotalMilliseconds;

            // Three more frames, unmeasured, whose pictures are hashed, so a prototype can be checked picture for picture against production.
            var pictures = new List<string>();
            var pixels = new byte[c.WindowWidth * c.WindowHeight * 4];
            for (int i = 0; i < 3; i++)
            {
                int n = c.Warmup + c.Frames + i;
                control.UpdateFrame(frames[n % frames.Length], c.SourceWidth, c.SourceHeight, c.RowRepeat, raw: raws[n % frames.Length]);
                using (var op = control.CaptureDrawOp(size)!) op.RenderTo(target.Canvas, context);
                context.Flush();
                gl.Finish();
                fixed (byte* p = pixels) target.ReadPixels(new SKImageInfo(c.WindowWidth, c.WindowHeight, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, c.WindowWidth * 4, 0, 0);
                pictures.Add(Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(pixels))[..12]);
            }
            SlangProbe.Current = null;
            control.ActiveSlangPreset = null;
            probe.Dispose();

            var stats = GameFrameControlStatistics(control);
            var line = new StringBuilder($"pace={c.Pace} kind={c.Kind} shader={Path.GetFileNameWithoutExtension(c.Shader).Replace(' ', '_')} console={(c.Console == "" ? "-" : c.Console)} set={(c.Set == "" ? "-" : c.Set)} raw={(c.Raw == "" ? "-" : c.Raw)} src={c.SourceWidth}x{c.SourceHeight}x{c.RowRepeat} window={c.WindowWidth}x{c.WindowHeight} out={Letterbox(c, shape)} shown={stats}");
            Summarise(rows, line, log);
            line.Append(CultureInfo.InvariantCulture, $" alloc.per.frame={bytes / (double)rows.Count:F0} gc={g0}/{g1}/{g2} gc.pause.ms={pauseMs:F1} pictures={string.Join(',', pictures)}");
            log($"allocated {bytes / (double)rows.Count / 1024:F1} KiB a frame; collections gen0/1/2 {g0}/{g1}/{g2}; pauses {pauseMs:F1} ms over {rows.Count} frames");
            return line.ToString();
        }

        // What a control draws on a GL device after each frame is shown once, as RGBA rows; a null frame is a redraw - see EmuSen_Serenity.md §3.9.
        public static byte[] Picture(GameFrameControl control, IReadOnlyList<byte[]?> frames, int sourceWidth, int sourceHeight, int rowRepeat, int windowWidth, int windowHeight, string? device, int lines = 0, RawPicture? raw = null)
        {
            using var gl = GlContext.Create(device);
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => GlContext.GetProc(name)) ?? throw new InvalidOperationException("no GL interface");
            using GRContext context = GRContext.CreateGl(glInterface) ?? throw new InvalidOperationException("no GRContext");
            using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(windowWidth, windowHeight, SKColorType.Rgba8888, SKAlphaType.Premul))
                ?? throw new InvalidOperationException("no GPU surface");
            var pixels = new byte[windowWidth * windowHeight * 4];
            control.WaitForFilter();
            foreach (byte[]? frame in frames)
            {
                if (frame is not null) control.UpdateFrame(frame, sourceWidth, sourceHeight, rowRepeat, lines: lines, raw: raw);
                target.Canvas.Clear(SKColors.Black);
                using (var op = control.CaptureDrawOp(new Size(windowWidth, windowHeight)) ?? throw new InvalidOperationException("no draw operation")) op.RenderTo(target.Canvas, context);
                context.Flush();
                gl.Finish();
            }
            fixed (byte* p = pixels) target.ReadPixels(new SKImageInfo(windowWidth, windowHeight, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, windowWidth * 4, 0, 0);
            control.ActiveFilter = null;
            return pixels;
        }

        private static string GameFrameControlStatistics(GameFrameControl control)
        {
            var s = control.TakeStatistics();
            return $"{s.Width}x{s.Height}";
        }

        // The rectangle the picture is drawn in: the frame's own shape, or the shape a filter states (EmuSen_Serenity.md §3.9).
        private static string Letterbox(Case c, double? aspect)
        {
            double shapeWidth = c.SourceWidth, shapeHeight = c.SourceHeight * c.RowRepeat;
            if (aspect is { } shape) shapeWidth = shapeHeight * shape;
            var (_, _, w, h) = GameFrameControl.ComputeLetterboxRect(shapeWidth, shapeHeight, c.WindowWidth, c.WindowHeight);
            return $"{Math.Round(w)}x{Math.Round(h)}";
        }

        internal static double Ms(long ticks) => ticks * 1000.0 / Stopwatch.Frequency;

        // The medians, means and 95th percentiles of each stage, written to the log and appended to the result line.
        internal static void Summarise(List<Dictionary<string, double>> rows, StringBuilder line, Action<string> log)
        {
            var keys = rows.SelectMany(r => r.Keys).Distinct().ToList();
            log($"{"stage",-22} {"median",8} {"mean",8} {"p95",8}   (ms, {rows.Count} frames)");
            foreach (string key in keys)
            {
                double[] v = rows.Select(r => r.GetValueOrDefault(key, double.NaN)).Where(x => !double.IsNaN(x)).OrderBy(x => x).ToArray();
                if (v.Length == 0) continue;
                double median = v[v.Length / 2], mean = v.Average(), p95 = v[(int)(v.Length * 0.95)];
                log($"{key,-22} {median,8:F3} {mean,8:F3} {p95,8:F3}");
                line.Append(CultureInfo.InvariantCulture, $" {key}={median:F3}/{mean:F3}");
            }
        }

        // The device path of EmuSen_DevicePicture.md §4 and §13 without a window: the chain presented into exported images, and the GL context standing in for Avalonia's compositor - see EmuSen_Serenity.md §13.
        internal static class DevicePath
        {
            private const uint Texture2D = 0x0DE1, Rgba8 = 0x8058, OpaqueFd = 0x9586, LayoutTransferSource = 0x9593;

            // One exported image and its two semaphores, imported once into the GL context, as DevicePresenter's slots are into the compositor.
            private sealed class Slot(SlangShown shown, uint memory, uint texture, uint ready, uint free)
            {
                public readonly SlangShown Shown = shown;
                public readonly uint Memory = memory, Texture = texture, Ready = ready, Free = free;
                public bool Used;
            }

            public static string Run(Case c, Action<string> log)
            {
                using var gl = GlContext.Create(Environment.GetEnvironmentVariable("EMUSEN_BENCH_GL_DEVICE"));
                log($"gl: {gl.Renderer}");
                foreach (string extension in new[] { "GL_EXT_memory_object_fd", "GL_EXT_semaphore_fd" })
                    if (!gl.HasExtension(extension)) throw new InvalidOperationException($"the GL context has no {extension}");
                using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => GlContext.GetProc(name)) ?? throw new InvalidOperationException("no GL interface");
                using GRContext context = GRContext.CreateGl(glInterface) ?? throw new InvalidOperationException("no GRContext");
                using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(c.WindowWidth, c.WindowHeight, SKColorType.Rgba8888, SKAlphaType.Premul))
                    ?? throw new InvalidOperationException("no GPU surface");
                SlangVulkan device = SlangRunner.SharedDevice ?? throw new InvalidOperationException($"no Vulkan device: {SlangRunner.DeviceReport}");
                log($"device: {device.Name}");

                var build = Stopwatch.StartNew();
                var (chain, filter, repeat) = Chain(c, device);
                double buildMs = build.Elapsed.TotalMilliseconds;
                log($"built in {buildMs:F0} ms");
                using SlangChain owned = chain;

                // The rectangle the device path draws in: the letterbox, snapped to whole pixels as GameFrameControl.PresentOnDevice snaps it.
                double shapeWidth = c.SourceWidth, shapeHeight = c.SourceHeight * c.RowRepeat;
                if (filter?.Aspect is { } aspect) shapeWidth = shapeHeight * aspect;
                var (dx, dy, dw, dh) = GameFrameControl.ComputeLetterboxRect(shapeWidth, shapeHeight, c.WindowWidth, c.WindowHeight);
                var destination = SKRect.Create((float)Math.Round(dx), (float)Math.Round(dy), (float)Math.Round(dw), (float)Math.Round(dh));
                int pw = (int)destination.Width, ph = (int)destination.Height;
                int imageWidth = (pw + DevicePresenter.RowPixels - 1) / DevicePresenter.RowPixels * DevicePresenter.RowPixels;
                if (filter is not null) chain.SetParameters(Values(c, filter, destination.Left, destination.Top));

                var frames = Enumerable.Range(0, 8).Select(k => Pattern(c.SourceWidth, c.SourceHeight, k)).ToArray();
                byte[]?[] raws = Enumerable.Range(0, 8).Select(k => c.Raw == "nes" && filter?.RawEncoding is { } encoding ? RawImage(NesRaw(c.SourceWidth, c.SourceHeight, k)) : null).ToArray();
                // The images made and imported as DevicePresenter makes them at a new size, timed: what a resize costs before its first present.
                var slots = new Slot[DevicePresenter.Slots];
                var imports = Stopwatch.StartNew();
                for (int s = 0; s < slots.Length; s++) slots[s] = Import(gl, device, imageWidth, ph);
                double importMs = imports.Elapsed.TotalMilliseconds;
                var probe = new Probe();
                probe.Attach(device);
                SlangProbe.Current = probe;
                uint glQuery = gl.NewQuery();

                var cpu = new List<Dictionary<string, double>>();
                var gpu = new List<Dictionary<string, double>>();
                long allocated = 0; int gen0 = 0, gen1 = 0, gen2 = 0; TimeSpan pause = TimeSpan.Zero;
                double firstPresent = 0, firstConsumer = 0;
                uint shownTexture = 0;
                SKImage? shownImage = null;
                var clock = Stopwatch.StartNew();
                double next = 0;
                int total = c.Warmup + c.Frames + 3;
                var pictures = new List<string>();
                var pixels = new byte[c.WindowWidth * c.WindowHeight * 4];
                for (int i = 0; i < total; i++)
                {
                    if (c.Pace > 0 && i < c.Warmup + c.Frames)
                    {
                        next += 1000.0 / c.Pace;
                        double wait = next - clock.Elapsed.TotalMilliseconds;
                        if (wait > 1.5) System.Threading.Thread.Sleep((int)(wait - 1));
                        while (clock.Elapsed.TotalMilliseconds < next) { }
                    }
                    if (i == c.Warmup) { allocated = GC.GetTotalAllocatedBytes(true); gen0 = GC.CollectionCount(0); gen1 = GC.CollectionCount(1); gen2 = GC.CollectionCount(2); pause = GC.GetTotalPauseDuration(); }
                    bool fresh = i % Every == 0;
                    Slot slot = slots[i % slots.Length];
                    probe.Begin();

                    // What the UI thread does on the device path: the raw frame and the frame given to the chain, and the chain presented without a wait.
                    long a0 = GC.GetAllocatedBytesForCurrentThread();
                    long t0 = Stopwatch.GetTimestamp();
                    if (fresh)
                    {
                        int k = i / Every % frames.Length;
                        if (raws[k] is { } raw) chain.ReplaceLut(BuiltInPreset.RawTexture, raw, c.SourceWidth, c.SourceHeight);
                        chain.Advance(frames[k], c.SourceWidth, c.SourceHeight, repeat);
                    }
                    long t1 = Stopwatch.GetTimestamp();
                    chain.Present(slot.Shown, pw, ph, waitFree: slot.Used);
                    slot.Used = true;
                    long t2 = Stopwatch.GetTimestamp();
                    long a1 = GC.GetAllocatedBytesForCurrentThread();

                    // What Avalonia's compositor does with an update: wait on ready, copy the image into a texture of its own, signal free, draw the copy.
                    gl.BeginQuery(glQuery);
                    bool direct = c.Consumer == "direct";
                    uint copy = Consume(gl, slot, imageWidth, ph, c.Consumer == "kept" ? shownTexture : 0, direct);
                    context.ResetContext();
                    var backend = new GRBackendTexture(imageWidth, ph, false, new GRGlTextureInfo(Texture2D, copy, Rgba8));
                    SKImage image = SKImage.FromTexture(context, backend, GRSurfaceOrigin.TopLeft, SKColorType.Rgba8888, SKAlphaType.Opaque) ?? throw new InvalidOperationException("Skia refused the texture");
                    target.Canvas.DrawImage(image, SKRect.Create(0, 0, pw, ph), destination, new SKSamplingOptions(SKFilterMode.Nearest, SKMipmapMode.None));
                    context.Flush();
                    if (direct) { context.ResetContext(); Signal(slot); }
                    long t3 = Stopwatch.GetTimestamp();
                    gl.EndQuery();
                    gl.Finish();
                    long t4 = Stopwatch.GetTimestamp();
                    shownImage?.Dispose();
                    backend.Dispose();
                    if (!direct && copy != shownTexture && shownTexture != 0) gl.DeleteTexture(shownTexture);
                    (shownImage, shownTexture) = (image, direct ? 0 : copy);
                    context.ResetContext();
                    double glGpu = gl.QueryNanoseconds(glQuery) / 1e6;
                    if (i == 0) (firstPresent, firstConsumer) = (Ms(t2 - t0), Ms(t3 - t2));

                    var (row, device_) = probe.PresentRow(t0, t1, t2);
                    row["ui.alloc.bytes"] = a1 - a0;
                    row["consumer.cpu"] = Ms(t3 - t2);
                    row["consumer.finish.wait"] = Ms(t4 - t3);
                    row["consumer.gl.gpu"] = glGpu;
                    // A submission's timestamps are read when the next frame settles it, so each frame's GPU figures are the ones heard in the frame after.
                    if (i > c.Warmup && i <= c.Warmup + c.Frames && (Every == 1 || (i - 1) % Every != 0)) gpu.Add(device_);
                    if (i >= c.Warmup && i < c.Warmup + c.Frames && (Every == 1 || i % Every != 0)) cpu.Add(row);
                    if (i >= c.Warmup + c.Frames)
                    {
                        fixed (byte* p = pixels) target.ReadPixels(new SKImageInfo(c.WindowWidth, c.WindowHeight, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, c.WindowWidth * 4, 0, 0);
                        pictures.Add(Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(pixels))[..12]);
                        // EMUSEN_BENCH_DUMP=<prefix> keeps each hashed picture's rows, for a probe that compares two builds' pictures by value.
                        if (Environment.GetEnvironmentVariable("EMUSEN_BENCH_DUMP") is { Length: > 0 } dump) File.WriteAllBytes($"{dump}-{pictures.Count}.rgba", pixels);
                    }
                    if (i == c.Warmup + c.Frames - 1)
                    {
                        long bytes = GC.GetTotalAllocatedBytes(true) - allocated;
                        (allocated, gen0, gen1, gen2) = (bytes, GC.CollectionCount(0) - gen0, GC.CollectionCount(1) - gen1, GC.CollectionCount(2) - gen2);
                        pause = GC.GetTotalPauseDuration() - pause;
                    }
                }
                SlangProbe.Current = null;
                probe.Dispose();
                shownImage?.Dispose();
                if (shownTexture != 0) gl.DeleteTexture(shownTexture);
                foreach (Slot slot in slots) Release(gl, slot);

                for (int i = 0; i < cpu.Count && i < gpu.Count; i++) foreach (var (key, value) in gpu[i]) cpu[i][key] = value;
                var line = new StringBuilder($"pace={c.Pace} path=present consumer={c.Consumer} kind={c.Kind} shader={System.IO.Path.GetFileNameWithoutExtension(c.Shader).Replace(' ', '_')} console={(c.Console == "" ? "-" : c.Console)} set={(c.Set == "" ? "-" : c.Set)} raw={(c.Raw == "" ? "-" : c.Raw)} src={c.SourceWidth}x{c.SourceHeight}x{c.RowRepeat} window={c.WindowWidth}x{c.WindowHeight} out={pw}x{ph}");
                Summarise(cpu, line, log);
                line.Append(CultureInfo.InvariantCulture, $" build.ms={buildMs:F0} import.ms={importMs:F2} first.present={firstPresent:F2} first.consumer={firstConsumer:F2} alloc.per.frame={allocated / (double)Math.Max(1, cpu.Count):F0} gc={gen0}/{gen1}/{gen2} gc.pause.ms={pause.TotalMilliseconds:F1} pictures={string.Join(',', pictures)}");
                log($"allocated {allocated / (double)Math.Max(1, cpu.Count) / 1024:F1} KiB a frame; first present {firstPresent:F2} ms, its consumer {firstConsumer:F2} ms");
                return line.ToString();
            }

            // The chain GameFrameControl would draw the case with, built as BuiltInFor and SlangRunner build it, and the repeat it hands the rows with.
            private static (SlangChain Chain, ScreenFilter? Filter, int Repeat) Chain(Case c, SlangVulkan device)
            {
                if (c.Kind == "slang") return (new SlangChain(device, SlangPreset.Load(c.Shader), SlangRunner.Cache), null, 1);
                ScreenFilter filter = c.Kind switch
                {
                    "filter" => c.Shader == CrtFilter.Name ? CrtFilter.Filter : ScreenFilters.Find(c.Shader.Replace('_', ' ')).Filter ?? throw new ArgumentException($"no filter {c.Shader}"),
                    "effect" => BuiltInShaders.AsFilter(ScreenFilters.Find(c.Shader.Replace('_', ' ')).Effect, false),
                    "none" => BuiltInShaders.AsFilter(ShaderEffect.None, false),
                    _ => throw new ArgumentException(c.Kind),
                };
                string? console = c.Console == "" ? null : c.Console;
                Dictionary<string, float>? structural = BuiltInPreset.StructuralOf(filter, Chosen(c));
                int by = BuiltInPreset.ReduceBy(filter, c.SourceWidth, c.SourceHeight * c.RowRepeat, 0);
                var chain = new SlangChain(device, SlangPreset.Load(BuiltInPreset.Write(filter, console, structural, by)), SlangRunner.Cache);
                return (chain, filter, filter.RowsOnce ? 1 : c.RowRepeat);
            }

            private static Dictionary<string, float>? Chosen(Case c) => c.Set == "" ? null : c.Set.Split(',').Select(p => p.Split(':')).ToDictionary(p => p[0], p => float.Parse(p[1], CultureInfo.InvariantCulture));

            // The values GameFrameControl.GiveValues hands a built-in's chain: each parameter's, the picture's corner, no crop, and the raw frame's size.
            private static Dictionary<string, float> Values(Case c, ScreenFilter filter, float x, float y)
            {
                Dictionary<string, float>? chosen = Chosen(c);
                string? console = c.Console == "" ? null : c.Console;
                var values = new Dictionary<string, float>(StringComparer.Ordinal);
                foreach (SlangParameter parameter in filter.Parameters ?? Array.Empty<SlangParameter>())
                    values[parameter.Id] = chosen is not null && chosen.TryGetValue(parameter.Id, out float value) ? value : filter.DefaultFor(parameter, console);
                (values[BuiltInPreset.OriginX], values[BuiltInPreset.OriginY]) = (x, y);
                bool raw = c.Raw == "nes" && filter.RawEncoding is not null;
                (values[BuiltInPreset.RawWidth], values[BuiltInPreset.RawHeight]) = (raw ? c.SourceWidth : 0, raw ? c.SourceHeight : 0);
                (values[BuiltInPreset.CropLeft], values[BuiltInPreset.CropTop], values[BuiltInPreset.CropWidth], values[BuiltInPreset.CropHeight]) = (0, 0, 1, 1);
                return values;
            }

            private static byte[]? RawImage(RawPicture raw)
            {
                var rgba = new byte[raw.Width * raw.Height * 4];
                return RawEncodings.ToImage(raw, rgba) ? rgba : null;
            }

            // An image exported by the device and imported into the GL context as Avalonia imports it: not dedicated, its tiling left to the driver.
            private static Slot Import(GlContext gl, SlangVulkan device, int width, int height)
            {
                SlangShown shown = SlangShown.TryCreate(device, width, height, linear: true, out string? problem) ?? throw new InvalidOperationException(problem);
                uint memory = 0, texture = 0, ready = 0, free = 0;
                ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glCreateMemoryObjectsEXT"))(1, &memory);
                ((delegate* unmanaged<uint, ulong, uint, int, void>)GlContext.GetProc("glImportMemoryFdEXT"))(memory, shown.MemoryBytes, OpaqueFd, shown.TakeMemoryFd());
                ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glGenTextures"))(1, &texture);
                ((delegate* unmanaged<uint, uint, void>)GlContext.GetProc("glBindTexture"))(Texture2D, texture);
                ((delegate* unmanaged<uint, int, uint, int, int, uint, ulong, void>)GlContext.GetProc("glTexStorageMem2DEXT"))(Texture2D, 1, Rgba8, width, height, memory, 0);
                var gen = (delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glGenSemaphoresEXT");
                var import = (delegate* unmanaged<uint, uint, int, void>)GlContext.GetProc("glImportSemaphoreFdEXT");
                gen(1, &ready);
                import(ready, OpaqueFd, shown.TakeReadyFd());
                gen(1, &free);
                import(free, OpaqueFd, shown.TakeFreeFd());
                if (gl.Error() is var error and not 0) throw new InvalidOperationException($"GL error 0x{error:X} importing the image");
                return new Slot(shown, memory, texture, ready, free);
            }

            // The compositor's part of an update: waits on ready, copies the image into a new texture (or into the one given), signals free; returns the copy. Direct draws the imported image itself, as the GL-context import of §8.5.1 would, and signals free after the draw.
            private static uint Consume(GlContext gl, Slot slot, int width, int height, uint into, bool direct)
            {
                Wait(slot);
                if (direct) return slot.Texture;
                uint copy = into;
                if (copy == 0)
                {
                    ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glGenTextures"))(1, &copy);
                    ((delegate* unmanaged<uint, uint, void>)GlContext.GetProc("glBindTexture"))(Texture2D, copy);
                    ((delegate* unmanaged<uint, int, uint, int, int, void>)GlContext.GetProc("glTexStorage2D"))(Texture2D, 1, Rgba8, width, height);
                }
                ((delegate* unmanaged<uint, uint, int, int, int, int, uint, uint, int, int, int, int, int, int, int, void>)GlContext.GetProc("glCopyImageSubData"))(
                    slot.Texture, Texture2D, 0, 0, 0, 0, copy, Texture2D, 0, 0, 0, 0, width, height, 1);
                Signal(slot);
                return copy;
            }

            private static void Wait(Slot slot)
            {
                uint texture = slot.Texture, layout = LayoutTransferSource;
                ((delegate* unmanaged<uint, uint, uint*, uint, uint*, uint*, void>)GlContext.GetProc("glWaitSemaphoreEXT"))(slot.Ready, 0, null, 1, &texture, &layout);
            }

            private static void Signal(Slot slot)
            {
                uint texture = slot.Texture, layout = LayoutTransferSource;
                ((delegate* unmanaged<uint, uint, uint*, uint, uint*, uint*, void>)GlContext.GetProc("glSignalSemaphoreEXT"))(slot.Free, 0, null, 1, &texture, &layout);
            }

            private static void Release(GlContext gl, Slot slot)
            {
                uint memory = slot.Memory, ready = slot.Ready, free = slot.Free;
                gl.DeleteTexture(slot.Texture);
                ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glDeleteSemaphoresEXT"))(1, &ready);
                ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glDeleteSemaphoresEXT"))(1, &free);
                ((delegate* unmanaged<int, uint*, void>)GlContext.GetProc("glDeleteMemoryObjectsEXT"))(1, &memory);
                gl.Finish();
                slot.Shown.Dispose();
            }
        }

        // The device the shared runner made, which the probe's timestamps must be written on.
        private static SlangVulkan? RunnerDevice()
        {
            var field = typeof(SlangRunner).GetField("Shared", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!;
            var lazy = (Lazy<(SlangVulkan? Device, string Report)>)field.GetValue(null)!;
            return lazy.Value.Device;
        }

        // Every colour number in moving bands, with the emphasis bits stepping down the picture, in the core ABI's nes.ppu.
        public static RawPicture NesRaw(int w, int h, int k)
        {
            var values = new byte[w * h * 2];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int v = ((x + k) / 4 + y / 30 * 8) % 64 | (y / 30 % 8) << 6, o = (y * w + x) * 2;
                    (values[o], values[o + 1]) = ((byte)v, (byte)(v >> 8));
                }
            return new RawPicture(RawEncodings.NesPpu, w, h, 2, w * 2, values);
        }

        // A picture that changes every frame, so no stage can skip work: a gradient under a moving checker.
        public static byte[] Pattern(int w, int h, int k)
        {
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++)
                for (int x = 0; x < w; x++)
                {
                    int o = (y * w + x) * 4;
                    bool check = (((x + k) >> 3) + (y >> 3) & 1) == 0;
                    rgba[o] = (byte)(x * 255 / Math.Max(1, w - 1));
                    rgba[o + 1] = (byte)(y * 255 / Math.Max(1, h - 1));
                    rgba[o + 2] = (byte)(check ? 220 : 40);
                    rgba[o + 3] = 255;
                }
            return rgba;
        }

        // Timestamps on the slang device and the CPU's phases, gathered per frame.
        private sealed class Probe : ISlangProbe, IDisposable
        {
            private const int PerSubmission = 64, Submissions = 4;
            private Vk? _vk;
            private Device _device;
            private QueryPool _pool;
            private double _period;
            private int _submission;
            private readonly List<(int Submission, int Slot, int Point)> _written = new();
            private readonly Dictionary<int, long> _phase = new();
            private readonly List<(int Submission, long Submitted, long Waited)> _waits = new();
            private readonly Dictionary<(int, int), double> _gpu = new();
            private long _pendingSubmitted;
            private SlangVulkan? _library;

            // The library's device times itself and its marks are handed on; the C#'s is marked from here.
            public void Attach(SlangVulkan gpu)
            {
                if (gpu.IsNative)
                {
                    _library = gpu;
                    gpu.SetProbe(true);
                    return;
                }
                _vk = gpu.Vk;
                _device = gpu.Device;
                var info = new QueryPoolCreateInfo { SType = StructureType.QueryPoolCreateInfo, QueryType = QueryType.Timestamp, QueryCount = PerSubmission * Submissions };
                SlangVulkan.Check(_vk.CreateQueryPool(_device, &info, null, out _pool), "vkCreateQueryPool");
                _period = TimestampPeriod(gpu);
            }

            private static double TimestampPeriod(SlangVulkan gpu)
            {
                Instance instance = gpu.Instance;
                uint count = 0;
                gpu.Vk.EnumeratePhysicalDevices(instance, &count, null);
                var devices = new PhysicalDevice[count];
                fixed (PhysicalDevice* p = devices) gpu.Vk.EnumeratePhysicalDevices(instance, &count, p);
                foreach (PhysicalDevice physical in devices)
                {
                    gpu.Vk.GetPhysicalDeviceProperties(physical, out PhysicalDeviceProperties properties);
                    if (Marshal.PtrToStringAnsi((nint)properties.DeviceName) == gpu.Name) return properties.Limits.TimestampPeriod;
                }
                throw new InvalidOperationException("the device's physical device was not found");
            }

            public void Begin()
            {
                _submission = 0;
                _written.Clear();
                _phase.Clear();
                _waits.Clear();
                _gpu.Clear();
            }

            public void Mark(CommandBuffer commands, int point)
            {
                if (_vk is null || _submission >= Submissions) return;
                int baseSlot = _submission * PerSubmission;
                if (point == SlangProbe.SubmissionBegin)
                {
                    _vk.CmdResetQueryPool(commands, _pool, (uint)baseSlot, PerSubmission);
                    _vk.CmdWriteTimestamp(commands, PipelineStageFlags.TopOfPipeBit, _pool, (uint)baseSlot);
                    _written.Add((_submission, baseSlot, point));
                    return;
                }
                int slot = baseSlot + (point == SlangProbe.SubmissionEnd ? PerSubmission - 1 : point + 1);
                if (point >= PerSubmission - 2) return;
                _vk.CmdWriteTimestamp(commands, PipelineStageFlags.BottomOfPipeBit, _pool, (uint)slot);
                _written.Add((_submission, slot, point));
            }

            public void Phase(int point) => Phase(point, Stopwatch.GetTimestamp());

            public void Gpu(int point, double milliseconds)
            {
                if (_submission < Submissions) _gpu[(_submission, point)] = milliseconds;
            }

            public void Phase(int point, long now)
            {
                if (point == SlangProbe.Submitted) { _pendingSubmitted = now; return; }
                if (point == SlangProbe.Waited)
                {
                    _waits.Add((_submission, _pendingSubmitted, now));
                    Read(_submission);
                    _submission++;
                    return;
                }
                _phase[point] = now;
            }

            private void Read(int submission)
            {
                if (_vk is null) return;
                var mine = _written.Where(w => w.Submission == submission).ToList();
                if (mine.Count == 0) return;
                ulong begin = 0;
                var values = new Dictionary<int, ulong>();
                foreach (var (_, slot, point) in mine)
                {
                    ulong value = 0;
                    _vk.GetQueryPoolResults(_device, _pool, (uint)slot, 1, sizeof(ulong), &value, sizeof(ulong), QueryResultFlags.Result64Bit);
                    values[point] = value;
                    if (point == SlangProbe.SubmissionBegin) begin = value;
                }
                foreach (var (point, value) in values) _gpu[(submission, point)] = (value - begin) * _period / 1e6;
            }

            // Stages in milliseconds; a frame with no slang work has only its CPU phases.
            public Dictionary<string, double> Row(long t0, long t1)
            {
                var row = new Dictionary<string, double> { ["render.thread.total"] = Ms(t1 - t0) };
                long P(int p) => _phase.TryGetValue(p, out long v) ? v : 0;
                if (P(SlangProbe.FlushBegin) != 0) { row["skia.flush"] = Ms(P(SlangProbe.FlushEnd) - P(SlangProbe.FlushBegin)); }
                if (P(SlangProbe.AdvanceBegin) != 0)
                {
                    row["source.copy"] = Ms(P(SlangProbe.AdvanceBegin) - t0);
                    row["advance.total"] = Ms(P(SlangProbe.AdvanceEnd) - P(SlangProbe.AdvanceBegin));
                }
                else if (P(SlangProbe.FlushBegin) != 0) row["record"] = Ms(P(SlangProbe.FlushBegin) - t0);
                if (P(SlangProbe.RenderBegin) != 0)
                {
                    row["render.size.bind"] = Ms(P(SlangProbe.Bound) - P(SlangProbe.RenderBegin));
                    row["render.host.copy"] = Ms(P(SlangProbe.Copied) - P(SlangProbe.Ran));
                    row["runner.image.copy"] = Ms(P(SlangProbe.ImageMade) - P(SlangProbe.Copied));
                    row["runner.drawimage"] = Ms(P(SlangProbe.FlushBegin) - P(SlangProbe.ImageMade));
                }
                foreach (var (submission, submitted, waited) in _waits)
                {
                    bool chain = _waits.Count > 1 ? submission == _waits.Count - 1 : P(SlangProbe.RenderBegin) != 0;
                    string name = chain ? "chain" : "upload";
                    row[$"{name}.wait"] = Ms(waited - submitted);
                    if (_gpu.TryGetValue((submission, SlangProbe.SubmissionEnd), out double end)) row[$"{name}.gpu"] = end;
                    if (!chain) continue;
                    int passes = _gpu.Keys.Where(k => k.Item1 == submission && k.Item2 >= 0).Select(k => k.Item2).DefaultIfEmpty(-1).Max();
                    if (passes >= 0 && _gpu.TryGetValue((submission, passes), out double lastPass))
                    {
                        row["chain.gpu.passes"] = lastPass;
                        row["chain.gpu.readback"] = end - lastPass;
                        double previous = 0;
                        for (int p = 0; p <= passes; p++)
                        {
                            if (!_gpu.TryGetValue((submission, p), out double at)) continue;
                            row[$"pass{p:D2}.gpu"] = at - previous;
                            previous = at;
                        }
                    }
                }
                return row;
            }

            // The device path's frame: its CPU stages, and the GPU figures of the submission settled in it, which is the frame before's present.
            public (Dictionary<string, double> Cpu, Dictionary<string, double> Gpu) PresentRow(long t0, long t1, long t2)
            {
                long P(int p) => _phase.TryGetValue(p, out long v) ? v : 0;
                var cpu = new Dictionary<string, double> { ["ui.total"] = Ms(t2 - t0), ["advance.total"] = Ms(t1 - t0), ["present.total"] = Ms(t2 - t1) };
                if (P(SlangProbe.RenderBegin) != 0 && P(SlangProbe.Bound) != 0) cpu["present.size.bind"] = Ms(P(SlangProbe.Bound) - P(SlangProbe.RenderBegin));
                if (P(SlangProbe.Bound) != 0 && P(SlangProbe.Ran) != 0) cpu["present.record.submit"] = Ms(P(SlangProbe.Ran) - P(SlangProbe.Bound));
                var gpu = new Dictionary<string, double>();
                if (_gpu.TryGetValue((0, SlangProbe.SubmissionEnd), out double end))
                {
                    gpu["present.gpu"] = end;
                    int passes = _gpu.Keys.Where(k => k.Item1 == 0 && k.Item2 >= 0).Select(k => k.Item2).DefaultIfEmpty(-1).Max();
                    double previous = 0;
                    for (int p = 0; p <= passes; p++)
                    {
                        if (!_gpu.TryGetValue((0, p), out double at)) continue;
                        gpu[$"pass{p:D2}.gpu"] = at - previous;
                        previous = at;
                    }
                    if (passes >= 0) { gpu["present.passes.gpu"] = previous; gpu["present.copy.gpu"] = end - previous; }
                }
                // A raw frame's lookup image is uploaded by a submission of its own, waited for on the UI thread.
                foreach (var (submission, submitted, waited) in _waits.Where(w => w.Submission >= 1))
                {
                    cpu[$"extra{submission}.wait"] = Ms(waited - submitted);
                    if (_gpu.TryGetValue((submission, SlangProbe.SubmissionEnd), out double extra)) cpu[$"extra{submission}.gpu"] = extra;
                }
                return (cpu, gpu);
            }

            public void Dispose()
            {
                if (_vk is not null && _pool.Handle != 0) _vk.DestroyQueryPool(_device, _pool, null);
                _vk = null;
                _library?.SetProbe(false);
                _library = null;
            }
        }

        // A surfaceless EGL context on a chosen device, desktop GL as Avalonia's GLX one is, with timer queries.
        internal sealed class GlContext : IDisposable
        {
            private const string Egl = "libEGL.so.1";
            [DllImport(Egl)] private static extern nint eglGetProcAddress(string name);
            [DllImport(Egl)] private static extern bool eglInitialize(nint display, out int major, out int minor);
            [DllImport(Egl)] private static extern bool eglBindAPI(uint api);
            [DllImport(Egl)] private static extern nint eglCreateContext(nint display, nint config, nint share, int[] attributes);
            [DllImport(Egl)] private static extern bool eglMakeCurrent(nint display, nint draw, nint read, nint context);
            [DllImport(Egl)] private static extern bool eglDestroyContext(nint display, nint context);
            [DllImport(Egl)] private static extern bool eglTerminate(nint display);

            private delegate* unmanaged<int, nint*, int*, bool> _queryDevices;
            private nint _display, _context;
            public string Renderer { get; private set; } = "";

            public static nint GetProc(string name) => eglGetProcAddress(name);

            public static GlContext Create(string? nameContains)
            {
                var queryDevices = (delegate* unmanaged<int, nint*, int*, bool>)eglGetProcAddress("eglQueryDevicesEXT");
                var platformDisplay = (delegate* unmanaged<uint, nint, int*, nint>)eglGetProcAddress("eglGetPlatformDisplayEXT");
                if (queryDevices == null || platformDisplay == null) throw new InvalidOperationException("EGL has no device enumeration");
                int count = 0;
                var devices = new nint[16];
                fixed (nint* d = devices) queryDevices(16, d, &count);
                var tried = new List<string>();
                for (int i = 0; i < count; i++)
                {
                    nint display = platformDisplay(0x313F, devices[i], null);
                    if (display == 0 || !eglInitialize(display, out _, out _)) continue;
                    eglBindAPI(0x30A2);
                    nint context = eglCreateContext(display, 0, 0, new[] { 0x3038 });
                    if (context == 0 || !eglMakeCurrent(display, 0, 0, context)) { eglTerminate(display); continue; }
                    var glGetString = (delegate* unmanaged<uint, byte*>)eglGetProcAddress("glGetString");
                    string renderer = Marshal.PtrToStringAnsi((nint)glGetString(0x1F01)) ?? "";
                    tried.Add(renderer);
                    bool software = renderer.Contains("llvmpipe", StringComparison.OrdinalIgnoreCase);
                    if ((string.IsNullOrEmpty(nameContains) && !software) || (!string.IsNullOrEmpty(nameContains) && renderer.Contains(nameContains, StringComparison.OrdinalIgnoreCase)))
                        return new GlContext { _display = display, _context = context, Renderer = renderer };
                    eglMakeCurrent(display, 0, 0, 0);
                    eglDestroyContext(display, context);
                }
                throw new InvalidOperationException($"no GL device matched \"{nameContains}\"; tried: {string.Join("; ", tried)}");
            }

            public uint NewQuery()
            {
                var gen = (delegate* unmanaged<int, uint*, void>)GetProc("glGenQueries");
                uint q = 0;
                gen(1, &q);
                return q;
            }

            public void BeginQuery(uint q) => ((delegate* unmanaged<uint, uint, void>)GetProc("glBeginQuery"))(0x88BF, q);
            public void EndQuery() => ((delegate* unmanaged<uint, void>)GetProc("glEndQuery"))(0x88BF);
            public void Finish() => ((delegate* unmanaged<void>)GetProc("glFinish"))();
            public uint Error() => ((delegate* unmanaged<uint>)GetProc("glGetError"))();

            public void DeleteTexture(uint texture) => ((delegate* unmanaged<int, uint*, void>)GetProc("glDeleteTextures"))(1, &texture);

            public bool HasExtension(string name)
            {
                int count = 0;
                ((delegate* unmanaged<uint, int*, void>)GetProc("glGetIntegerv"))(0x821D, &count);
                var getStringi = (delegate* unmanaged<uint, uint, byte*>)GetProc("glGetStringi");
                for (uint i = 0; i < count; i++) if (Marshal.PtrToStringAnsi((nint)getStringi(0x1F03, i)) == name) return true;
                return false;
            }

            public ulong QueryNanoseconds(uint q)
            {
                ulong v = 0;
                ((delegate* unmanaged<uint, uint, ulong*, void>)GetProc("glGetQueryObjectui64v"))(q, 0x8866, &v);
                return v;
            }

            public void Dispose()
            {
                eglMakeCurrent(_display, 0, 0, 0);
                eglDestroyContext(_display, _context);
            }
        }
    }
}
