using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text;
using Avalonia;
using EmuSen.Cores;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using SkiaSharp;

namespace EmuSen.WiseMan.Serenity
{
    // A game's frames from a state, run through the CRT and read back each frame, their mean light written as a table - see EmuSen_CRT.md §30.
    [Collection(EmuSen.WiseMan.Fixtures.TestCollections.ProcessGlobals)]
    public unsafe class CrtSceneProbe
    {
        public const string RomVariable = "EMUSEN_CRT_SCENE_ROM", StateVariable = "EMUSEN_CRT_SCENE_STATE", FramesVariable = "EMUSEN_CRT_SCENE_FRAMES";
        public const string PressVariable = "EMUSEN_CRT_SCENE_PRESS", OutVariable = "EMUSEN_CRT_SCENE_OUT", SetVariable = "EMUSEN_CRT_SCENE_SET", WindowVariable = "EMUSEN_CRT_SCENE_WINDOW";
        public const string ChainVariable = "EMUSEN_CRT_SCENE_CHAIN", SettledVariable = "EMUSEN_CRT_SCENE_SETTLED", ConsoleVariable = "EMUSEN_CRT_SCENE_CONSOLE";

        private static string ConsoleName => Environment.GetEnvironmentVariable(ConsoleVariable) ?? "SNES";

        private static readonly double[] ToLight = Enumerable.Range(0, 256).Select(i => i / 255.0).Select(c => c <= 0.04045 ? c / 12.92 : Math.Pow((c + 0.055) / 1.055, 2.4)).ToArray();

        private sealed record Frame(byte[] Rgba, int Width, int Height, int RowRepeat, int Lines);

        // Runs the game from the state with the presses (button:start:frames, comma-separated) and keeps each frame in a file beside the output.
        private static List<Frame> Capture(string rom, string state, int count, string presses, string cache)
        {
            if (File.Exists(cache)) return Read(cache);
            DataStore.OverrideDirectory = Path.Combine(Path.GetDirectoryName(cache)!, "home");
            var held = presses.Split(',', StringSplitOptions.RemoveEmptyEntries).Select(p => p.Split(':')).Select(p => (Button: Enum.Parse<PadButton>(p[0], true), Start: int.Parse(p[1]), Frames: int.Parse(p[2]))).ToArray();
            var frames = new List<Frame>();
            ICore core = CoreFactory.Loaded(rom, options: EmuSen.WiseMan.Fixtures.TestCores.NoBattery);
            try
            {
                core.LoadState(state);
                for (int i = 0; i < count; i++)
                {
                    foreach (var b in held.Select(h => h.Button).Distinct()) core.SetButton(0, b, held.Any(h => h.Button == b && i >= h.Start && i < h.Start + h.Frames));
                    core.RunFrame();
                    core.DequeueAudioSamples(int.MaxValue);
                    byte[] rgba = core.GetFrameBufferRgba();
                    int w = core.ScreenWidth, h = core.ScreenHeight;
                    frames.Add(new Frame(rgba[..(w * h * 4)].ToArray(), w, h, (core as IRepeatedRows)?.RowRepeat ?? 1, core.DisplayLines));
                }
            }
            finally
            {
                (core as IDisposable)?.Dispose();
                DataStore.OverrideDirectory = null;
            }
            using var writer = new BinaryWriter(File.Create(cache));
            writer.Write(frames.Count);
            foreach (Frame f in frames) { writer.Write(f.Width); writer.Write(f.Height); writer.Write(f.RowRepeat); writer.Write(f.Lines); writer.Write(f.Rgba.Length); writer.Write(f.Rgba); }
            return frames;
        }

        private static List<Frame> Read(string cache)
        {
            using var reader = new BinaryReader(File.OpenRead(cache));
            int n = reader.ReadInt32();
            var frames = new List<Frame>(n);
            for (int i = 0; i < n; i++)
            {
                int w = reader.ReadInt32(), h = reader.ReadInt32(), repeat = reader.ReadInt32(), lines = reader.ReadInt32();
                frames.Add(new Frame(reader.ReadBytes(reader.ReadInt32()), w, h, repeat, lines));
            }
            return frames;
        }

        // The largest difference in mean light, as a share of the settled picture's mean, over 16 by 16 blocks.
        private static double LargestBlock(byte[] a, byte[] b, int w, int h)
        {
            double largest = 0;
            for (int by = 0; by + 16 <= h; by += 16)
                for (int bx = 0; bx + 16 <= w; bx += 16)
                {
                    double da = MeanLight(a, bx, by, bx + 16, by + 16, w), db = MeanLight(b, bx, by, bx + 16, by + 16, w);
                    if (db > 0.02) largest = Math.Max(largest, Math.Abs(da - db) / db);
                }
            return largest;
        }

        // The largest difference in any colour's code value, and the share of them that differ at all.
        private static (double Largest, double Changed) Codes(byte[] a, byte[] b)
        {
            int largest = 0;
            long changed = 0;
            for (int i = 0; i < a.Length; i++)
            {
                if (i % 4 == 3) continue;
                int d = Math.Abs(a[i] - b[i]);
                largest = Math.Max(largest, d);
                if (d > 0) changed++;
            }
            return (largest, changed / (a.Length * 0.75));
        }

        private static double MeanLight(byte[] rgba, int x0, int y0, int x1, int y1, int stride)
        {
            double sum = 0;
            for (int y = y0; y < y1; y++)
                for (int x = x0; x < x1; x++)
                {
                    int i = (y * stride + x) * 4;
                    sum += 0.2126 * ToLight[rgba[i]] + 0.7152 * ToLight[rgba[i + 1]] + 0.0722 * ToLight[rgba[i + 2]];
                }
            return sum / Math.Max(1, (x1 - x0) * (y1 - y0));
        }

        private static readonly Avalonia.Headless.HeadlessUnitTestSession Session =
            Avalonia.Headless.HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CrtSceneProbe).Assembly);

        [Fact]
        public System.Threading.Tasks.Task A_scene_change_through_the_crt_written_as_a_table() => Session.Dispatch(Run, default);

        // The same frames through the chain alone, each config "label|id:value,...|native 0/1|present 0/1|renders a frame" separated by ';', with each frame's difference from the settled frames named.
        [Fact]
        public void A_scene_change_through_the_chain_alone_written_as_a_table()
        {
            string? rom = Environment.GetEnvironmentVariable(RomVariable), state = Environment.GetEnvironmentVariable(StateVariable), output = Environment.GetEnvironmentVariable(OutVariable);
            string? configs = Environment.GetEnvironmentVariable(ChainVariable);
            if (string.IsNullOrWhiteSpace(rom) || string.IsNullOrWhiteSpace(state) || string.IsNullOrWhiteSpace(output) || string.IsNullOrWhiteSpace(configs)) return;
            int count = int.Parse(Environment.GetEnvironmentVariable(FramesVariable) ?? "300");
            List<Frame> frames = Capture(rom, state, count, Environment.GetEnvironmentVariable(PressVariable) ?? "", Path.ChangeExtension(output, null) + ".frames");
            string[] size = (Environment.GetEnvironmentVariable(WindowVariable) ?? "1600x1200").Split('x');
            int vw = int.Parse(size[0]), vh = int.Parse(size[1]);
            var columns = new List<(string Label, double[] Light)>();
            foreach (string config in configs.Split(';', StringSplitOptions.RemoveEmptyEntries))
            {
                string[] c = config.Split('|');
                var chosen = new Dictionary<string, float>();
                foreach (string[] p in c[1].Split(',', StringSplitOptions.RemoveEmptyEntries).Select(p => p.Split(':'))) chosen[p[0]] = float.Parse(p[1], CultureInfo.InvariantCulture);
                bool native = c[2] == "1", present = c[3] == "1";
                int renders = int.Parse(c[4]);
                using EmuSen.Serenity.Slang.SlangVulkan gpu = EmuSen.Serenity.Slang.SlangVulkan.TryCreate(out string report, Environment.GetEnvironmentVariable(EmuSen.Serenity.Slang.SlangVulkan.DeviceVariable) ?? "6800", null, native) ?? throw new InvalidOperationException(report);
                string preset = BuiltInPreset.Write(CrtFilter.Filter, ConsoleName, chosen);
                var light = new double[frames.Count];
                int[] references = (Environment.GetEnvironmentVariable(SettledVariable) ?? $"{frames.Count - 1}").Split(',').Select(int.Parse).ToArray();
                var settled = new Dictionary<int, byte[]>();
                var block = new double[frames.Count];
                var largest = new double[frames.Count];
                var changed = new double[frames.Count];
                // Twice: once for each frame's light and the settled pictures, again for each frame's difference from the next settled one of its parity.
                for (int pass = 0; pass < 2; pass++)
                {
                    using var chain = new EmuSen.Serenity.Slang.SlangChain(gpu, EmuSen.Serenity.Slang.SlangPreset.Load(preset), null, 1);
                    using var shown = present ? EmuSen.Serenity.Slang.SlangShown.TryCreate(gpu, vw, vh, true, out string? problem) ?? throw new InvalidOperationException(problem) : null;
                    for (int i = 0; i < frames.Count; i++)
                    {
                        Frame f = frames[i];
                        chain.Advance(f.Rgba, f.Width, f.Height, 1);
                        if (shown is not null) chain.Present(shown, vw, vh, waitFree: false);
                        byte[] picture = chain.Render(vw, vh);
                        for (int k = 1; k < renders; k++) picture = chain.Render(vw, vh);
                        if (pass == 0)
                        {
                            light[i] = MeanLight(picture, 0, 0, vw, vh, vw);
                            if (references.Any(r => i == r || i == r - 1)) settled[i] = picture;
                        }
                        else if (references.Where(r => r >= i).DefaultIfEmpty(-1).First() is int r && r >= 0)
                        {
                            byte[] steady = settled[r - (r - i) % 2];
                            block[i] = LargestBlock(picture, steady, vw, vh);
                            (largest[i], changed[i]) = Codes(picture, steady);
                        }
                    }
                }
                columns.Add((c[0] + ".block", block));
                columns.Add((c[0] + ".code", largest));
                columns.Add((c[0] + ".changed", changed));
                columns.Add((c[0], light));
            }
            var table = new StringBuilder("frame\tcore\t" + string.Join("\t", columns.Select(c => c.Label)) + "\n");
            for (int i = 0; i < frames.Count; i++)
                table.Append(CultureInfo.InvariantCulture, $"{i}\t{MeanLight(frames[i].Rgba, 0, 0, frames[i].Width, frames[i].Height, frames[i].Width):F5}\t{string.Join("\t", columns.Select(c => c.Light[i].ToString("F5", CultureInfo.InvariantCulture)))}\n");
            File.WriteAllText(output, table.ToString());
        }

        private static void Run()
        {
            string? rom = Environment.GetEnvironmentVariable(RomVariable), state = Environment.GetEnvironmentVariable(StateVariable), output = Environment.GetEnvironmentVariable(OutVariable);
            if (string.IsNullOrWhiteSpace(rom) || string.IsNullOrWhiteSpace(state) || string.IsNullOrWhiteSpace(output)) return;
            int count = int.Parse(Environment.GetEnvironmentVariable(FramesVariable) ?? "300");
            string presses = Environment.GetEnvironmentVariable(PressVariable) ?? "";
            List<Frame> frames = Capture(rom, state, count, presses, Path.ChangeExtension(output, null) + ".frames");

            string set = Environment.GetEnvironmentVariable(SetVariable) ?? "";
            string[] size = (Environment.GetEnvironmentVariable(WindowVariable) ?? "1280x800").Split('x');
            int ww = int.Parse(size[0]), wh = int.Parse(size[1]);
            var control = new GameFrameControl();
            if (set != "off")
            {
                control.FilterConsole = ConsoleName;
                control.ShaderParameters = set == "" ? new Dictionary<string, float>() : set.Split(',').Select(p => p.Split(':')).ToDictionary(p => p[0], p => float.Parse(p[1], CultureInfo.InvariantCulture));
                control.ActiveFilter = CrtFilter.Filter;
                control.WaitForFilter();
            }

            using var gl = ShaderBench.GlContext.Create(Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) ?? "6800");
            using GRGlInterface glInterface = GRGlInterface.CreateOpenGl(name => ShaderBench.GlContext.GetProc(name))!;
            using GRContext context = GRContext.CreateGl(glInterface)!;
            using SKSurface target = SKSurface.Create(context, true, new SKImageInfo(ww, wh, SKColorType.Rgba8888, SKAlphaType.Premul))!;
            var pixels = new byte[ww * wh * 4];
            var (px, py, pw, ph) = GameFrameControl.ComputeLetterboxRect(4, 3, ww, wh);
            int x0 = (int)Math.Ceiling(px), y0 = (int)Math.Ceiling(py), x1 = (int)Math.Floor(px + pw), y1 = (int)Math.Floor(py + ph);

            var table = new StringBuilder("frame\tsize\tcore\tshown\tratio\n");
            for (int i = 0; i < frames.Count; i++)
            {
                Frame f = frames[i];
                control.UpdateFrame(f.Rgba, f.Width, f.Height, f.RowRepeat, lines: f.Lines);
                target.Canvas.Clear(SKColors.Black);
                using (var op = control.CaptureDrawOp(new Size(ww, wh))!) op.RenderTo(target.Canvas, context);
                context.Flush();
                gl.Finish();
                fixed (byte* p = pixels) target.ReadPixels(new SKImageInfo(ww, wh, SKColorType.Rgba8888, SKAlphaType.Premul), (nint)p, ww * 4, 0, 0);
                double core = MeanLight(f.Rgba, 0, 0, f.Width, f.Height, f.Width), shown = MeanLight(pixels, x0, y0, x1, y1, ww);
                table.Append(CultureInfo.InvariantCulture, $"{i}\t{f.Width}x{f.Height}x{f.RowRepeat}/{f.Lines}\t{core:F5}\t{shown:F5}\t{(core > 1e-5 ? shown / core : 0):F4}\n");
            }
            control.ActiveFilter = null;
            File.WriteAllText(output, table.ToString());
        }
    }
}
