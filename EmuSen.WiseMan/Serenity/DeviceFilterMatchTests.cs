using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using SkiaSharp;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Each built-in filter drawn by Skia from its SkSL and by the chain from its translation, compared channel by channel - see EmuSen_DevicePicture.md §3.3.
    [Collection(EmuSen.WiseMan.Fixtures.TestCollections.ProcessGlobals)]
    public class DeviceFilterMatchTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(DeviceFilterMatchTests).GetTypeInfo().Assembly);

        // A folder of real games' pictures (PNG), copied from the library; without it the synthetic frames stand in.
        public const string FramesVariable = "EMUSEN_GPU_FRAMES";

        private static readonly Lazy<string?> GlDevice = new(() =>
        {
            string name = Environment.GetEnvironmentVariable(ShaderBenchTests.GlVariable) ?? "6800";
            try { using var gl = ShaderBench.GlContext.Create(name); return name; }
            catch (Exception) { return null; }
        });

        private readonly ITestOutputHelper _output;
        private readonly SlangVulkan? _gpu;

        public DeviceFilterMatchTests(ITestOutputHelper output)
        {
            _output = output;
            _gpu = SlangVulkan.TryCreate(out string report, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800");
            _output.WriteLine($"device: {report}");
        }

        public void Dispose() => _gpu?.Dispose();

        // What two pictures differ by: the largest difference of a channel, how many channels differ at all and by more than one, and where the first few of those are.
        public sealed record Difference(int Largest, long Differing, long BeyondOne, long Channels, IReadOnlyList<(int X, int Y, int Channel, int Skia, int Device)> Worst)
        {
            // Channels beyond one whose Skia value the chain itself draws, within one, once its sampling points move by up to four Nudges: a discontinuity or a steep slope met within float error of the pixel's centre.
            public long AtTies { get; init; }

            public long Unexplained => BeyondOne - AtTies;

            public override string ToString() =>
                $"largest {Largest}, differing {Differing} of {Channels} ({100.0 * Differing / Channels:F4}%), beyond one {BeyondOne}, at ties {AtTies}" +
                (Worst.Count > 0 ? ": " + string.Join("; ", Worst.Take(6).Select(w => $"({w.X},{w.Y}) c{w.Channel} {w.Skia}/{w.Device}")) : "");
        }

        public static Difference Compare(byte[] skia, byte[] device, int width)
        {
            int largest = 0;
            long differing = 0, beyond = 0;
            var worst = new List<(int, int, int, int, int)>();
            for (int i = 0; i < skia.Length; i++)
            {
                if ((i & 3) == 3) continue;
                int d = Math.Abs(skia[i] - device[i]);
                if (d == 0) continue;
                differing++;
                if (d > 1) { beyond++; if (worst.Count < 32) worst.Add(((i / 4) % width, (i / 4) / width, i & 3, skia[i], device[i])); }
                largest = Math.Max(largest, d);
            }
            return new Difference(largest, differing, beyond, skia.Length / 4 * 3, worst);
        }

        // §3.3's tolerance as measured: every channel within one level, any beyond it at a tie, and of the rest no more than one in a million - see EmuSen_DevicePicture.md §12.
        public static bool WithinTolerance(Difference d) => d.Unexplained * 1_000_000 <= d.Channels;

        // How far the sampling points are moved to ask whether a difference is a tie: a thousandth of a pixel, then four, which no display shows.
        public const float Nudge = 1f / 1024;

        // The filter drawn both ways over the same frames; the chain is given each new frame and rendered once for it, as the runner does.
        private (byte[] Skia, byte[] Device)? Both(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen, IReadOnlyList<byte[]> frames,
            int width, int height, int rowRepeat, int windowWidth, int windowHeight, ShaderEffect? effect = null, RawPicture? raw = null, bool bilinear = true)
        {
            if (GlDevice.Value is not { } gl || _gpu is null) return null;
            byte[] skia = Session.Dispatch(() =>
                ShaderBench.Picture(effect is { } simple ? new GameFrameControl { ActiveEffect = simple, BilinearFiltering = bilinear } : new GameFrameControl { FilterConsole = console, ShaderParameters = chosen, ActiveFilter = filter, BilinearFiltering = bilinear },
                    frames.ToArray(), width, height, rowRepeat, windowWidth, windowHeight, gl, raw: raw),
                default).GetAwaiter().GetResult();

            return (skia, Device(filter, console, chosen, frames, width, height, rowRepeat, windowWidth, windowHeight, default, raw));
        }

        private byte[] Device(ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen, IReadOnlyList<byte[]> frames,
            int width, int height, int rowRepeat, int windowWidth, int windowHeight, (float X, float Y) nudge, RawPicture? raw = null)
        {
            string path = BuiltInPreset.Write(filter, console, chosen, nudge: nudge);
            using var chain = new SlangChain(_gpu!, SlangPreset.Load(path), SlangRunner.Cache);
            var values = chosen?.Where(v => filter.Parameters?.Any(p => p.Id == v.Key) == true).ToDictionary(v => v.Key, v => v.Value) ?? new Dictionary<string, float>();
            // The picture before the palette as the frame control hands it to the device's chain: the filter's image of it, and its size - see EmuSen_CRT.md §22.8.
            byte[]? image = null;
            if (raw is { } r && filter.RawEncoding == r.Encoding && RawEncodings.ToImage(r, image = new byte[r.Width * r.Height * 4]))
                (values[BuiltInPreset.RawWidth], values[BuiltInPreset.RawHeight]) = (r.Width, r.Height);
            chain.SetParameters(values);
            byte[] device = Array.Empty<byte>();
            foreach (byte[] frame in frames)
            {
                if (image is not null) chain.ReplaceLut(BuiltInPreset.RawTexture, image, raw!.Value.Width, raw.Value.Height);
                chain.Advance(frame, width, height, filter.RowsOnce ? 1 : rowRepeat);
                device = chain.Render(windowWidth, windowHeight);
            }
            return device;
        }

        private Difference? Match(string what, ScreenFilter filter, string? console, IReadOnlyDictionary<string, float>? chosen, IReadOnlyList<byte[]> frames,
            int width, int height, int rowRepeat, int windowWidth, int windowHeight, ShaderEffect? effect = null, RawPicture? raw = null, bool bilinear = true)
        {
            if (Both(filter, console, chosen, frames, width, height, rowRepeat, windowWidth, windowHeight, effect, raw, bilinear) is not var (skia, device)) return null;
            Difference d = Compare(skia, device, windowWidth);
            if (d.BeyondOne > 0)
            {
                // A channel beyond one is a tie when Skia's value lies within one of the values the chain draws there with its sampling points moved a little.
                var unexplained = new Dictionary<int, (int Low, int High)>();
                for (int i = 0; i < skia.Length; i++) if ((i & 3) != 3 && Math.Abs(skia[i] - device[i]) > 1) unexplained[i] = (device[i], device[i]);
                foreach ((float X, float Y) nudge in new[] { 1f, 4f }.SelectMany(k => new[] { (k * Nudge, 0f), (-k * Nudge, 0f), (0f, k * Nudge), (0f, -k * Nudge), (k * Nudge, k * Nudge), (-k * Nudge, -k * Nudge), (k * Nudge, -k * Nudge), (-k * Nudge, k * Nudge) }))
                {
                    if (unexplained.Count == 0) break;
                    byte[] moved = Device(filter, console, chosen, frames, width, height, rowRepeat, windowWidth, windowHeight, nudge, raw);
                    foreach (int i in unexplained.Keys.ToArray())
                    {
                        var (low, high) = unexplained[i];
                        (low, high) = (Math.Min(low, moved[i]), Math.Max(high, moved[i]));
                        if (skia[i] >= low - 1 && skia[i] <= high + 1) unexplained.Remove(i);
                        else unexplained[i] = (low, high);
                    }
                }
                d = d with { AtTies = d.BeyondOne - unexplained.Count };
                foreach (var (i, (low, high)) in unexplained.Take(8)) _output.WriteLine($"  unexplained ({i / 4 % windowWidth},{i / 4 / windowWidth}) c{i & 3}: skia {skia[i]} device {device[i]}, moved {low} to {high}");
            }
            _output.WriteLine($"{what}: {d}");
            return d;
        }

        // A frame whose every edge and level differs: a gradient under a checker of four sizes.
        public static byte[] Pattern(int width, int height, int seed)
        {
            var rgba = new byte[width * height * 4];
            for (int y = 0; y < height; y++)
                for (int x = 0; x < width; x++)
                {
                    int i = (y * width + x) * 4;
                    bool check = ((x >> (1 + seed % 3)) + (y >> (1 + (seed + 1) % 3))) % 2 == 0;
                    rgba[i] = (byte)(check ? x * 255 / Math.Max(1, width - 1) : 255 - y * 255 / Math.Max(1, height - 1));
                    rgba[i + 1] = (byte)((x * 7 + y * 13 + seed * 50) & 0xFF);
                    rgba[i + 2] = (byte)(check ? 40 + seed * 30 : 220 - seed * 20);
                    rgba[i + 3] = 255;
                }
            return rgba;
        }

        // Real games' pictures of a size from the folder named by the variable, each as RGBA; none without it.
        public static IReadOnlyList<(string Name, byte[] Rgba)> RealFrames(int width, int height)
        {
            string? folder = Environment.GetEnvironmentVariable(FramesVariable);
            if (string.IsNullOrEmpty(folder) || !Directory.Exists(folder)) return Array.Empty<(string, byte[])>();
            var frames = new List<(string, byte[])>();
            foreach (string file in Directory.GetFiles(folder, "*.png").OrderBy(f => f, StringComparer.Ordinal))
            {
                using SKBitmap? decoded = SKBitmap.Decode(file);
                if (decoded is null || decoded.Width != width || decoded.Height != height) continue;
                using var rgba = new SKBitmap(new SKImageInfo(width, height, SKColorType.Rgba8888, SKAlphaType.Unpremul));
                using (var canvas = new SKCanvas(rgba)) { canvas.Clear(SKColors.Black); canvas.DrawBitmap(decoded, 0, 0); }
                byte[] bytes = rgba.GetPixelSpan().ToArray();
                for (int i = 3; i < bytes.Length; i += 4) bytes[i] = 255;
                frames.Add((Path.GetFileNameWithoutExtension(file), bytes));
            }
            return frames;
        }

        // Four synthetic frames, then each real one shown after them, so that a filter reading history reads frames that differ.
        private static IEnumerable<(string Name, byte[][] Frames)> Sequences(int width, int height)
        {
            byte[][] synthetic = Enumerable.Range(0, 4).Select(seed => Pattern(width, height, seed)).ToArray();
            yield return ("synthetic", synthetic);
            foreach (var (name, rgba) in RealFrames(width, height)) yield return (name, synthetic.Append(rgba).ToArray());
        }

        public static IEnumerable<object[]> Handhelds() =>
            new[] { HandheldFilters.DmgLcd, HandheldFilters.PocketLcd, HandheldFilters.LightLcd, HandheldFilters.GbcLcd, HandheldFilters.AgbLcd, HandheldFilters.Ags101Lcd }
                .Select(f => new object[] { f.Name });

        [Theory]
        [MemberData(nameof(Handhelds))]
        public void A_handheld_lcd_on_the_chain_draws_what_skia_draws(string name)
        {
            ScreenFilter filter = ScreenFilters.Find(name).Filter!;
            foreach (int scale in new[] { 2, 5 })
                foreach (var (what, frames) in Sequences(160, 144))
                {
                    if (Match($"{name} {what} x{scale}", filter, "GB", null, frames, 160, 144, 1, 160 * scale, 144 * scale) is not { } d) return;
                    Assert.True(WithinTolerance(d), $"{name} {what} x{scale}: {d}");
                }
        }
    

        // Every size a console's frame takes, with the window it is drawn in whole: width, rows, repeat, window.
        private static readonly (string Console, int Width, int Height, int Repeat, int WindowWidth, int WindowHeight)[] Frames =
        {
            ("NES", 256, 240, 1, 1024, 960), ("NES", 256, 240, 1, 768, 720),
            ("SNES", 256, 224, 1, 1024, 896), ("SNES", 256, 224, 1, 1000, 875),
            ("Genesis", 320, 224, 1, 1280, 896),
            ("N64", 640, 240, 2, 1280, 960), ("N64", 640, 480, 1, 1440, 1080), ("N64", 640, 288, 2, 1200, 1080),
        };

        [Fact]
        public void Crt_lottes_on_the_chain_draws_what_skia_draws()
        {
            foreach (var (console, width, height, repeat, ww, wh) in Frames.Where(f => Environment.GetEnvironmentVariable("EMUSEN_MATCH_ONLY") is not { } only || f.Console == only))
                foreach (var (what, frames) in Sequences(width, height))
                {
                    if (Match($"Lottes {console} {width}x{height}x{repeat} in {ww}x{wh} {what}", CrtFilters.Lottes, console, null, frames, width, height, repeat, ww, wh) is not { } d) return;
                    Assert.True(WithinTolerance(d), $"Lottes {console} {width}x{height} {what}: {d}");
                }
        }

        [Theory]
        [InlineData(ShaderEffect.None, false)]
        [InlineData(ShaderEffect.None, true)]
        [InlineData(ShaderEffect.Scanlines, false)]
        [InlineData(ShaderEffect.Scanlines, true)]
        [InlineData(ShaderEffect.Crt, false)]
        [InlineData(ShaderEffect.Crt, true)]
        public void The_plain_picture_and_the_simple_effects_on_the_chain_draw_what_skia_draws(ShaderEffect effect, bool bilinear)
        {
            foreach (var (console, width, height, repeat, ww, wh) in Frames)
                foreach (var (what, frames) in Sequences(width, height))
                {
                    string name = $"{effect} {(bilinear ? "bilinear" : "nearest")} {console} {width}x{height}x{repeat} in {ww}x{wh} {what}";
                    if (Match(name, BuiltInShaders.AsFilter(effect, bilinear), console, null, frames, width, height, repeat, ww, wh, effect, bilinear: bilinear) is not { } d) return;
                    Assert.True(WithinTolerance(d), $"{name}: {d}");
                }
        }
    

        // The NES's own signal on the device: a nes.ppu picture of every colour under each emphasis setting, drawn by the chain as Skia draws it - see EmuSen_CRT.md §22.8.
        [Theory]
        [MemberData(nameof(CrtVariants))]
        public void The_crt_on_the_chain_draws_the_nes_s_raw_picture_as_skia_does(int quality, int signal) => RawNes(quality, signal, pal: false);

        // The same on PAL, where the 2C07's signal is drawn - see EmuSen_CRT.md §27.
        [Theory]
        [MemberData(nameof(CrtVariants))]
        public void The_crt_on_the_chain_draws_the_pal_nes_s_raw_picture_as_skia_does(int quality, int signal) => RawNes(quality, signal, pal: true);

        private void RawNes(int quality, int signal, bool pal)
        {
            var chosen = new Dictionary<string, float> { ["quality"] = quality, ["signal"] = signal, ["standard"] = pal ? 1 : 0 };
            RawPicture raw = ScreenFilterRenderTests.RawField(256, 240, (x, y) => (y / 30 * 8 + x / 32, y / 30 % 8));
            byte[] blank = new byte[256 * 240 * 4];
            for (int i = 3; i < blank.Length; i += 4) blank[i] = 255;
            string name = $"CRT q{quality} s{signal} {(pal ? "PAL " : "")}NES from nes.ppu";
            if (Match(name, CrtFilter.Filter, "NES", chosen, new[] { blank, blank }, 256, 240, 1, 1440, 1080, raw: raw) is not { } d) return;
            Assert.True(WithinTolerance(d), $"{name}: {d}");
        }

        // The CRT's frames by console: a frame of each kind a core hands over, drawn into the 4:3 tube filling a 1440 by 1080 window.
        private static readonly (string Console, int Width, int Height, int Repeat)[] CrtFrames =
        {
            ("NES", 256, 240, 1), ("SNES", 256, 224, 1), ("SNES", 512, 448, 1), ("Genesis", 320, 224, 1),
            ("N64", 640, 240, 2), ("N64", 640, 288, 2), ("N64", 640, 480, 1), ("PlayStation", 320, 240, 1), ("PlayStation", 640, 480, 1),
        };

        public static IEnumerable<object[]> CrtVariants() =>
            from quality in new[] { 0, 1, 2 } from signal in new[] { 0, 1, 2 } select new object[] { quality, signal };

        [Theory]
        [MemberData(nameof(CrtVariants))]
        public void The_crt_on_the_chain_draws_what_skia_draws(int quality, int signal)
        {
            var chosen = new Dictionary<string, float> { ["quality"] = quality, ["signal"] = signal };
            foreach (var (console, width, height, repeat) in CrtFrames.Where(f => Environment.GetEnvironmentVariable("EMUSEN_MATCH_ONLY") is not { } only || f.Console == only))
                foreach (var (what, frames) in Sequences(width, height))
                {
                    string name = $"CRT q{quality} s{signal} {console} {width}x{height}x{repeat} {what}";
                    if (Match(name, CrtFilter.Filter, console, chosen, frames, width, height, repeat, 1440, 1080) is not { } d) return;
                    Assert.True(WithinTolerance(d), $"{name}: {d}");
                }
        }
    }
}
