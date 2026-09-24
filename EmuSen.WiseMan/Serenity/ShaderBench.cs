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
        public sealed record Case(string Kind, string Shader, int SourceWidth, int SourceHeight, int RowRepeat, int WindowWidth, int WindowHeight, int Frames, int Warmup, double Pace);

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
                double.Parse(d.GetValueOrDefault("pace", "60"), CultureInfo.InvariantCulture));
        }

        // A preset's build, whole and in parts, after a small preset has warmed the compiler and the device.
        public static string Load(Case c, Action<string> log)
        {
            using SlangVulkan gpu = SlangVulkan.TryCreate(out string report) ?? throw new InvalidOperationException(report);
            log($"device: {report}");
            string stock = Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(c.Shader))!, "stock.slang");
            if (!File.Exists(stock)) stock = Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(Path.GetDirectoryName(Path.GetDirectoryName(c.Shader))))!, "stock.slang");
            if (File.Exists(stock)) { var warm = SlangSource.Load(stock); SlangCompiler.Compile(warm.Vertex, SlangStage.Vertex, stock); }

            var clock = Stopwatch.StartNew();
            SlangPreset preset = SlangPreset.Load(c.Shader);
            double presetMs = clock.Elapsed.TotalMilliseconds;
            clock.Restart();
            using (var chain = new SlangChain(gpu, preset)) { }
            double chainMs = clock.Elapsed.TotalMilliseconds;

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
            string line = FormattableString.Invariant($"kind=load shader={Path.GetFileNameWithoutExtension(c.Shader)} passes={preset.Passes.Count} luts={preset.Textures.Count} load.chain={chainMs:F0} load.preset={presetMs:F1} load.source={sourceMs:F0} load.compile={compileMs:F0} load.lut={lutMs:F0} load.rest={chainMs - sourceMs - compileMs - lutMs:F0}");
            log(line);
            return line;
        }

        // One case: returns a line of name=value medians, and writes the per-stage table to the log.
        public static string Run(Case c, Action<string> log)
        {
            if (c.Kind == "load") return Load(c, log);
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
                case "filter": control.ActiveFilter = ScreenFilters.Find(c.Shader.Replace('_', ' ')).Filter ?? throw new ArgumentException($"no filter {c.Shader}"); break;
                case "effect": control.ActiveEffect = ScreenFilters.Find(c.Shader.Replace('_', ' ')).Effect; break;
                case "none": break;
                default: throw new ArgumentException(c.Kind);
            }

            var frames = Enumerable.Range(0, 8).Select(k => Pattern(c.SourceWidth, c.SourceHeight, k)).ToArray();
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
                if (i % Every == 0) control.UpdateFrame(frames[i / Every % frames.Length], c.SourceWidth, c.SourceHeight, c.RowRepeat);
                using var op = control.CaptureDrawOp(size) ?? throw new InvalidOperationException("no draw operation");
                probe.Begin();
                gl.BeginQuery(glQuery);
                long t0 = Stopwatch.GetTimestamp();
                op.RenderTo(target.Canvas, context);
                long t1 = Stopwatch.GetTimestamp();
                gl.EndQuery();
                context.ResetContext();
                gl.Finish();
                long t2 = Stopwatch.GetTimestamp();
                double glGpu = gl.QueryNanoseconds(glQuery) / 1e6;
                context.ResetContext();
                if (!measured || (Every > 1 && i % Every == 0)) continue;
                var row = probe.Row(t0, t1);
                row["gl.finish.wait"] = Ms(t2 - t1);
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
                control.UpdateFrame(frames[n % frames.Length], c.SourceWidth, c.SourceHeight, c.RowRepeat);
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
            var keys = rows.SelectMany(r => r.Keys).Distinct().ToList();
            var line = new StringBuilder($"pace={c.Pace} kind={c.Kind} shader={Path.GetFileNameWithoutExtension(c.Shader).Replace(' ', '_')} src={c.SourceWidth}x{c.SourceHeight}x{c.RowRepeat} window={c.WindowWidth}x{c.WindowHeight} out={Letterbox(c)} shown={stats}");
            log($"{"stage",-22} {"median",8} {"mean",8} {"p95",8}   (ms, {rows.Count} frames)");
            foreach (string key in keys)
            {
                double[] v = rows.Select(r => r.GetValueOrDefault(key, double.NaN)).Where(x => !double.IsNaN(x)).OrderBy(x => x).ToArray();
                if (v.Length == 0) continue;
                double median = v[v.Length / 2], mean = v.Average(), p95 = v[(int)(v.Length * 0.95)];
                log($"{key,-22} {median,8:F3} {mean,8:F3} {p95,8:F3}");
                line.Append(CultureInfo.InvariantCulture, $" {key}={median:F3}/{mean:F3}");
            }
            line.Append(CultureInfo.InvariantCulture, $" alloc.per.frame={bytes / (double)rows.Count:F0} gc={g0}/{g1}/{g2} gc.pause.ms={pauseMs:F1} pictures={string.Join(',', pictures)}");
            log($"allocated {bytes / (double)rows.Count / 1024:F1} KiB a frame; collections gen0/1/2 {g0}/{g1}/{g2}; pauses {pauseMs:F1} ms over {rows.Count} frames");
            return line.ToString();
        }

        private static string GameFrameControlStatistics(GameFrameControl control)
        {
            var s = control.TakeStatistics();
            return $"{s.Width}x{s.Height}";
        }

        private static string Letterbox(Case c)
        {
            var (_, _, w, h) = GameFrameControl.ComputeLetterboxRect(c.SourceWidth, c.SourceHeight * c.RowRepeat, c.WindowWidth, c.WindowHeight);
            return $"{Math.Round(w)}x{Math.Round(h)}";
        }

        private static double Ms(long ticks) => ticks * 1000.0 / Stopwatch.Frequency;

        // The device the shared runner made, which the probe's timestamps must be written on.
        private static SlangVulkan? RunnerDevice()
        {
            var field = typeof(SlangRunner).GetField("Shared", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!;
            var lazy = (Lazy<(SlangVulkan? Device, string Report)>)field.GetValue(null)!;
            return lazy.Value.Device;
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

            public void Attach(SlangVulkan gpu)
            {
                _vk = gpu.Vk;
                _device = gpu.Device;
                var info = new QueryPoolCreateInfo { SType = StructureType.QueryPoolCreateInfo, QueryType = QueryType.Timestamp, QueryCount = PerSubmission * Submissions };
                SlangVulkan.Check(_vk.CreateQueryPool(_device, &info, null, out _pool), "vkCreateQueryPool");
                _period = TimestampPeriod(gpu);
            }

            private static double TimestampPeriod(SlangVulkan gpu)
            {
                var instance = (Instance)typeof(SlangVulkan).GetField("_instance", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance)!.GetValue(gpu)!;
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

            public void Phase(int point)
            {
                long now = Stopwatch.GetTimestamp();
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

            public void Dispose()
            {
                if (_vk is not null && _pool.Handle != 0) _vk.DestroyQueryPool(_device, _pool, null);
                _vk = null;
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
