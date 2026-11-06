using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Crystal;

namespace EmuSen.WiseMan.Serenity
{
    // Every preset the built-in filters write, and the catalog's facts, held to what the C# wrote before the writing moved to the platform library - see EmuSen_Backend.md §40.6.
    public sealed class FilterPresetParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFilterPresets", Guid.NewGuid().ToString("N"));

        public FilterPresetParityTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static readonly string?[] Consoles =
            { null, "", "NES", "SNES", "Genesis", "Sega CD", "32X", "N64", "PlayStation", "PlayStation 2", "Master System", "SG-1000", "GB", "GBC", "GBA", "Saturn" };

        private static readonly (float X, float Y) Nudge = (0.25f, -0.125f);

        private static readonly float[] Chromas = { 0.5f, 0.8f, 1.37f, 3f };

        private const int RandomStructural = 12, RandomAll = 6, RandomOther = 5;

        // The filters a preset is written for: the catalog's, then the simple effects nearest and linear.
        private static IReadOnlyList<(string Key, ScreenFilter Filter)> Filters() =>
            ScreenFilters.All.Where(c => c.Filter is not null).Select(c => (c.Name, c.Filter!))
                .Concat(new[] { ShaderEffect.None, ShaderEffect.Scanlines, ShaderEffect.Crt }.SelectMany(e => new[] { false, true }.Select(b => ($"effect {e} {(b ? "linear" : "nearest")}", BuiltInShaders.AsFilter(e, b)))))
                .ToList();

        private static string Bits(float value) => BitConverter.SingleToUInt32Bits(value).ToString("X8", CultureInfo.InvariantCulture);

        private static string Shown(string? console) => console is null ? "(none)" : console.Length == 0 ? "(any)" : console;

        private static string Values(IReadOnlyDictionary<string, float>? values) =>
            values is null ? "null" : string.Join(",", values.OrderBy(v => v.Key, StringComparer.Ordinal).Select(v => $"{v.Key}={Bits(v.Value)}"));

        private static string Hash(byte[] bytes) => Convert.ToHexString(SHA256.HashData(bytes))[..16];

        // A written preset as its folder's name, the files beside it and the raw frame's stand-in.
        private string Written(Func<string> write)
        {
            string path;
            try { path = write(); }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
            string folder = Path.GetDirectoryName(path)!;
            Assert.Equal(_root, Path.GetDirectoryName(folder));
            Assert.Equal("filter.slangp", Path.GetFileName(path));
            string[] files = Directory.GetFiles(folder).Select(Path.GetFileName).OrderBy(f => f, StringComparer.Ordinal).ToArray()!;
            string png = File.Exists(Path.Combine(folder, "raw.png")) ? Hash(File.ReadAllBytes(Path.Combine(folder, "raw.png"))) : "-";
            return $"{Path.GetFileName(folder)} files={string.Join(" ", files)} png={png} preset={Hash(File.ReadAllBytes(path))}";
        }

        private static Dictionary<string, float> Random(Random random, IEnumerable<SlangParameter> parameters, bool wide)
        {
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (SlangParameter p in parameters)
            {
                float low = wide ? p.Minimum - 1 : p.Minimum, high = wide ? p.Maximum + 1 : p.Maximum;
                values[p.Id] = p.Choices is not null ? MathF.Round(low + (float)random.NextDouble() * (high - low)) : low + (float)random.NextDouble() * (high - low);
            }
            return values;
        }

        [Fact]
        public void Every_preset_a_built_in_filter_writes_is_the_one_recorded()
        {
            var golden = new ParityGolden("FilterPresetParity.Writes");
            var filters = Filters();
            var random = new Random(20261105);
            int cases = 0;
            foreach (var (key, filter) in filters)
            {
                IReadOnlyList<SlangParameter> parameters = filter.Parameters ?? Array.Empty<SlangParameter>();
                bool crt = filter.Structural is not null;
                foreach (string? console in Consoles)
                {
                    void Write(string what, IReadOnlyDictionary<string, float>? chosen, int by = 1, (float X, float Y) nudge = default)
                    {
                        golden.Add($"{key} | {Shown(console)} | {what} | by={by}{(nudge == default ? "" : " nudged")}", Written(() => BuiltInPreset.WriteInto(_root, filter, console, chosen, by, nudge)));
                        cases++;
                    }

                    Write("default", null);
                    Write("default", null, 1, Nudge);
                    Write("default", null, 2);
                    Write("default", null, 4);
                    if (crt)
                    {
                        foreach (SlangParameter p in parameters.Where(p => filter.Structural!.Contains(p.Id) && p.Choices is not null))
                            for (int v = (int)p.Minimum; v <= (int)p.Maximum; v++)
                                Write($"{p.Id}={v}", new Dictionary<string, float> { [p.Id] = v });
                        foreach (float chroma in Chromas) Write($"chroma={Bits(chroma)}", new Dictionary<string, float> { ["chroma"] = chroma });
                        for (int i = 0; i < RandomStructural; i++)
                        {
                            var chosen = BuiltInPreset.StructuralOf(filter, Random(random, parameters, wide: i % 3 == 2))!;
                            Write($"structural {Values(chosen)}", chosen);
                        }
                        for (int i = 0; i < RandomAll; i++)
                        {
                            var chosen = Random(random, parameters, wide: false);
                            Write($"all {Values(chosen)}", chosen, i % 2 == 0 ? 1 : 3);
                        }
                    }
                    else
                    {
                        for (int i = 0; i < RandomOther; i++)
                        {
                            var chosen = Random(random, parameters, wide: i == RandomOther - 1);
                            Write($"all {Values(chosen)}", chosen, 1, i == 0 ? Nudge : default);
                        }
                    }
                }
            }
            ScreenFilter structured = filters.Single(f => f.Filter.Structural is not null).Filter;
            int choiceRuns = structured.Parameters!.Where(p => structured.Structural!.Contains(p.Id) && p.Choices is not null).Sum(p => (int)(p.Maximum - p.Minimum) + 1);
            int crtFilters = filters.Count(f => f.Filter.Structural is not null);
            int least = Consoles.Length * (filters.Count * 4 + crtFilters * (choiceRuns + Chromas.Length + RandomStructural + RandomAll) + (filters.Count - crtFilters) * RandomOther);
            Assert.Equal(least, cases);
            golden.Check(least);
        }

        [Fact]
        public void The_catalog_says_what_it_said()
        {
            var golden = new ParityGolden("FilterPresetParity.Catalog");
            var filters = Filters();
            foreach (var (key, f) in filters)
            {
                golden.Add($"{key} name", f.Name);
                golden.Add($"{key} consoles", f.Consoles is null ? "null" : string.Join("|", f.Consoles));
                golden.Add($"{key} credit", f.Credit);
                foreach (SlangParameter p in f.Parameters ?? Array.Empty<SlangParameter>())
                    golden.Add($"{key} parameter {p.Id}", $"{p.Description}|{Bits(p.Initial)}|{Bits(p.Minimum)}|{Bits(p.Maximum)}|{Bits(p.Step)}|{(p.Choices is null ? "-" : string.Join(";", p.Choices))}");
                golden.Add($"{key} structural", f.Structural is null ? "null" : string.Join("|", f.Structural));
                golden.Add($"{key} raw", f.RawEncoding ?? "null");
                golden.Add($"{key} shape", $"aspect={(f.Aspect is { } a ? BitConverter.DoubleToInt64Bits(a).ToString("X16", CultureInfo.InvariantCulture) : "null")} crops={f.CropsItself} rowsOnce={f.RowsOnce} depth={f.HistoryDepth} history={f.History}");
                foreach (var (console, values) in (f.ConsoleDefaults ?? new Dictionary<string, IReadOnlyDictionary<string, float>>()).OrderBy(c => c.Key, StringComparer.Ordinal))
                    golden.Add($"{key} defaults {Shown(console)}", Values(values));
            }
            foreach (ConsoleScreens.Screen s in ConsoleScreens.All)
                golden.Add($"screen {s.Console}", $"{s.Kind}|{s.Profile}|{s.HostsPicture}|{s.Gap ?? "-"}");
            foreach (ConsoleScreens.Owed o in ConsoleScreens.OwedRows) golden.Add($"owed {o.Console}", o.Note);
            golden.Add("televisions", string.Join("|", ConsoleScreens.Televisions));
            golden.Add("names", string.Join("|", ScreenFilters.Names));
            foreach (string? console in Consoles.Where(c => c is not null)) golden.Add($"names for {Shown(console)}", string.Join("|", ScreenFilters.NamesFor(console!)));
            foreach (string? name in new[] { null, "", "crt", "CRT", "Simple CRT", "None", "Game Boy LCD", "Nothing" })
            {
                ScreenFilterChoice c = ScreenFilters.Find(name);
                golden.Add($"find {name ?? "(null)"}", $"{c.Name}|{c.Effect}|{c.Filter?.Name ?? "-"}");
            }
            golden.Check(filters.Count * 6);
        }

        [Fact]
        public void Values_raw_reads_reductions_and_structural_choices_are_what_they_were()
        {
            var golden = new ParityGolden("FilterPresetParity.Values");
            var filters = Filters();
            var random = new Random(51102026);
            int answers = 0;
            foreach (var (key, f) in filters)
            {
                IReadOnlyList<SlangParameter> parameters = f.Parameters ?? Array.Empty<SlangParameter>();
                foreach (string? console in Consoles)
                {
                    for (int i = 0; i < 4; i++)
                    {
                        Dictionary<string, float>? given = i == 0 ? null : Random(random, parameters, wide: i == 3);
                        if (i == 2 && given is not null) given["standard"] = 1;
                        golden.AddChunked($"{key} values", $"{Shown(console)} {Values(given)}: {Values(f.ValuesFor(console, given))} raw={f.ReadsRaw(console, given)} written={Values(BuiltInPreset.ValuesFor(f, console, given))} structural={Values(BuiltInPreset.StructuralOf(f, given))}", 16);
                        answers++;
                    }
                    foreach (SlangParameter p in parameters) golden.AddChunked($"{key} defaults", $"{Shown(console)} {p.Id}={Bits(f.DefaultFor(p, console))}", 64);
                    answers += parameters.Count;
                }
                foreach (int width in new[] { 256, 320, 512, 640, 768, 1280 })
                    foreach (int height in new[] { 224, 240, 448, 480, 896, 960, 1920 })
                        foreach (int lines in new[] { 0, 1, 112, 224, 240, 239 })
                        {
                            golden.AddChunked($"{key} reduce", $"{width}x{height}/{lines}={BuiltInPreset.ReduceBy(f, width, height, lines)}", 64);
                            answers++;
                        }
            }
            Assert.Equal(filters.Count * (Consoles.Length * 4 + 6 * 7 * 6) + filters.Sum(f => (f.Filter.Parameters?.Count ?? 0) * Consoles.Length), answers);
            golden.Check(filters.Count * 3);
        }
    }
}
