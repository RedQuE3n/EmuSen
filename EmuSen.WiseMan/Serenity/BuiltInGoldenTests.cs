using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Serenity
{
    // Every variant of every built-in filter drawn by the chain and hashed, held to hashes recorded on the RX 6800 - see EmuSen_DevicePicture.md §19.
    [Collection(EmuSen.WiseMan.Fixtures.TestCollections.ProcessGlobals)]
    public class BuiltInGoldenTests : IDisposable
    {
        public const string GoldenName = "BuiltInGoldens.txt";
        public const string RecordVariable = "EMUSEN_RECORD_GOLDENS";

        private readonly ITestOutputHelper _output;
        private readonly SlangVulkan? _gpu;
        private readonly string _report;

        public BuiltInGoldenTests(ITestOutputHelper output)
        {
            _output = output;
            _gpu = SlangVulkan.TryCreate(out _report, Environment.GetEnvironmentVariable(SlangVulkan.DeviceVariable) ?? "6800");
            _output.WriteLine($"device: {_report}");
        }

        public void Dispose() => _gpu?.Dispose();

        // One variant: the filter, its console and chosen values, the frame it is given, the window it fills, and the multiple a frame is brought back from.
        public sealed record Variant(string Name, ScreenFilter Filter, string? Console, IReadOnlyDictionary<string, float> Chosen, int Width, int Height, int RowRepeat,
            int WindowWidth, int WindowHeight, int ReduceBy = 1, bool Raw = false);

        private static readonly (string Console, int Width, int Height, int Repeat)[] CrtFrames =
        {
            ("NES", 256, 240, 1), ("SNES", 256, 224, 1), ("Genesis", 320, 224, 1), ("N64", 640, 240, 2), ("PlayStation", 320, 240, 1),
        };

        // The 90 of the CRT's tier, signal and standard on each console, the NES's from its raw picture, the reduced frames, Lottes, the six LCDs and the simple effects.
        public static IEnumerable<Variant> Variants()
        {
            foreach (var (console, width, height, repeat) in CrtFrames)
                foreach (int standard in new[] { 0, 1 })
                    for (int quality = 0; quality < 3; quality++)
                        for (int signal = 0; signal < 3; signal++)
                            yield return new Variant($"crt {console} std{standard} q{quality} s{signal}", CrtFilter.Filter, console,
                                new Dictionary<string, float> { ["quality"] = quality, ["signal"] = signal, ["standard"] = standard }, width, height, repeat, 800, 600);
            foreach (int standard in new[] { 0, 1 })
                for (int quality = 0; quality < 3; quality++)
                    for (int signal = 0; signal < 3; signal++)
                        yield return new Variant($"crt NES raw std{standard} q{quality} s{signal}", CrtFilter.Filter, "NES",
                            new Dictionary<string, float> { ["quality"] = quality, ["signal"] = signal, ["standard"] = standard }, 256, 240, 1, 800, 600, Raw: true);
            for (int quality = 0; quality < 3; quality++)
                yield return new Variant($"crt N64 reduced q{quality}", CrtFilter.Filter, "N64", new Dictionary<string, float> { ["quality"] = quality, ["standard"] = 0 }, 640, 480, 1, 800, 600, ReduceBy: 2);
            yield return new Variant("lottes SNES", CrtFilters.Lottes, "SNES", new Dictionary<string, float>(), 256, 224, 1, 1024, 896);
            yield return new Variant("lottes N64", CrtFilters.Lottes, "N64", new Dictionary<string, float>(), 640, 240, 2, 1280, 960);
            foreach (ScreenFilter lcd in new[] { HandheldFilters.DmgLcd, HandheldFilters.PocketLcd, HandheldFilters.LightLcd, HandheldFilters.GbcLcd, HandheldFilters.AgbLcd, HandheldFilters.Ags101Lcd })
                yield return new Variant($"lcd {lcd.Name}", lcd, "GB", new Dictionary<string, float>(), 160, 144, 1, 480, 432);
            foreach (ShaderEffect effect in new[] { ShaderEffect.None, ShaderEffect.Scanlines, ShaderEffect.Crt })
                foreach (bool bilinear in new[] { false, true })
                    yield return new Variant($"effect {effect} {(bilinear ? "bilinear" : "nearest")}", BuiltInShaders.AsFilter(effect, bilinear), null, new Dictionary<string, float>(), 256, 224, 1, 768, 672);
        }

        // The chain given four frames of a moving pattern, rendered once for each as the runner renders, and its last picture.
        public static byte[] Render(SlangVulkan gpu, Variant v)
        {
            string path = BuiltInPreset.Write(v.Filter, v.Console, v.Chosen, v.ReduceBy);
            using var chain = new SlangChain(gpu, SlangPreset.Load(path), SlangRunner.Cache);
            var values = v.Chosen.Where(c => v.Filter.Parameters?.Any(p => p.Id == c.Key) == true).ToDictionary(c => c.Key, c => c.Value);
            byte[]? image = null;
            RawPicture raw = ScreenFilterRenderTests.RawField(v.Width, v.Height, (x, y) => (y / 30 * 8 + x / 32, y / 30 % 8));
            if (v.Raw && RawEncodings.ToImage(raw, image = new byte[v.Width * v.Height * 4])) (values[BuiltInPreset.RawWidth], values[BuiltInPreset.RawHeight]) = (v.Width, v.Height);
            chain.SetParameters(values);
            byte[] picture = Array.Empty<byte>();
            for (int seed = 0; seed < 4; seed++)
            {
                if (image is not null) chain.ReplaceLut(BuiltInPreset.RawTexture, image, v.Width, v.Height);
                chain.Advance(Pattern(v.Width, v.Height, seed), v.Width, v.Height, v.Filter.RowsOnce ? 1 : v.RowRepeat);
                picture = chain.Render(v.WindowWidth, v.WindowHeight);
            }
            return picture;
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

        public static string Hash(byte[] picture) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(picture))[..16].ToLowerInvariant();

        [Fact]
        public void Every_variant_of_every_built_in_filter_draws_its_recorded_picture()
        {
            if (_gpu is null) return;
            var actual = Variants().Select(v => $"{v.Name}\t{Hash(Render(_gpu, v))}").ToList();
            string golden = Path.Combine(AppContext.BaseDirectory, "Serenity", GoldenName);
            if (Environment.GetEnvironmentVariable(RecordVariable) == "1")
            {
                File.WriteAllLines(Path.Combine(Path.GetDirectoryName(SourceFile())!, GoldenName), new[] { $"# {_report}" }.Concat(actual));
                return;
            }
            var expected = File.ReadAllLines(golden).Where(l => !l.StartsWith('#')).ToList();
            var differing = actual.Except(expected).ToList();
            Assert.True(expected.Count == actual.Count && differing.Count == 0,
                $"{differing.Count} of {actual.Count} variants differ from {GoldenName} ({expected.Count} recorded):" + string.Concat(differing.Take(40).Select(l => "\n  " + l)));
        }

        private static string SourceFile([CallerFilePath] string path = "") => path;
    }
}
