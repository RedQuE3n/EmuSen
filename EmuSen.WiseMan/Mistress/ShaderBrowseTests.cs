using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // A big pack in Settings ▸ Display ▸ Shaders: a preset of hundreds of parameters on its page, their search, the pad reaching every row, and the parameters kept for the presets read last - see EmuSen_BigPicture.md §52.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderBrowseTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderBrowseTests).GetTypeInfo().Assembly);

        // A Mega Bezel SMOOTH-ADV's count: 953 declared, nine of them headings.
        private const int Big = 944;
        private const int Headings = 9;
        private const int Row = 12;

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderBrowseTests", Guid.NewGuid().ToString("N"));
        private readonly string _pack;

        public ShaderBrowseTests(ITestOutputHelper output)
        {
            _out = output;
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _pack = SlangPackDownload.DefaultDirectory;
            Write("shaders/glow.slang", Shader(new[] { ("GLOW", "Glow strength", 0.25, 0.0, 1.0, 0.05), ("SCAN", "Scanline weight", 3.0, 1.0, 8.0, 1.0) }));
            WriteBig(_pack);
            for (int i = 0; i < Row; i++) Write($"row/p{i:D2}.slangp", "shaders = 1\nshader0 = ../shaders/glow.slang\n");
            Write(SlangPackDownload.StampFile, "2026-09-22 02:00 UTC");
        }

        // big/huge.slangp: 944 parameters under nine headings, named P0001 onwards, each 0.5 in 0 to 1 by 0.05.
        internal static void WriteBig(string pack)
        {
            var many = new List<(string, string, double, double, double, double)>();
            for (int i = 0; i < Big + Headings; i++)
                many.Add(i % 106 == 0 ? ($"HEAD{i}", $"--- Section {i / 106} ---", 0, 0, 0, 1) : ($"P{i:D4}", $"Parameter {i:D4}", 0.5, 0.0, 1.0, 0.05));
            foreach ((string relative, string text) in new[] { ("shaders/many.slang", Shader(many)), ("big/huge.slangp", "shaders = 1\nshader0 = ../shaders/many.slang\n") })
            {
                Directory.CreateDirectory(Path.GetDirectoryName(Path.Combine(pack, relative))!);
                File.WriteAllText(Path.Combine(pack, relative), text);
            }
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void Write(string relative, string text)
        {
            string path = Path.Combine(_pack, relative);
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllText(path, text);
        }

        private static string Shader(IEnumerable<(string Id, string Description, double Initial, double Minimum, double Maximum, double Step)> parameters)
        {
            var text = new StringBuilder("#version 450\n");
            foreach (var p in parameters)
                text.Append(FormattableString.Invariant($"#pragma parameter {p.Id} \"{p.Description}\" {p.Initial:F2} {p.Minimum:F2} {p.Maximum:F2} {p.Step:F2}\n"));
            text.Append("#pragma stage vertex\nvoid main() { gl_Position = vec4(0.0); }\n#pragma stage fragment\nvoid main() { }\n");
            return text.ToString();
        }

        private const string Huge = "slang:big/huge.slangp";

        // The SNES's page with the big preset in use, and its parameters page open.
        private static MainWindow HugeParameters()
        {
            MainWindow window = SettingsDesk.Open(SettingsScreen.Display, "shaders", "shader:SNES", "folder:big");
            SettingsDesk.Choose(window, "use:" + Huge);
            SettingsDesk.Back(window);
            SettingsDesk.Choose(window, "params");
            return window;
        }

        private static IEnumerable<string> Parameters(MainWindow window) => SettingsDesk.Keys(window).Where(k => k.StartsWith("p:", StringComparison.Ordinal));

        private static void SearchParameters(MainWindow window, string words)
        {
            if (window.SettingsShown!.Trail[^1] != "Parameters") SettingsDesk.Back(window);
            SettingsDesk.Choose(window, "paramsearch");
            SettingsDesk.Answer(window, words);
        }

        [Fact]
        public Task A_944_parameter_preset_is_a_page_per_heading_and_its_last_parameter_reached_and_moved() => Session.Dispatch(() =>
        {
            MainWindow window = HugeParameters();
            string[] sections = SettingsDesk.Keys(window).Where(k => k.StartsWith("sec:", StringComparison.Ordinal)).ToArray();
            Assert.Equal(Enumerable.Range(0, Headings).Select(i => $"sec:HEAD{i * 106}"), sections);
            Assert.Equal(Big, sections.Sum(k => int.Parse(SettingsDesk.Value(window, k)!)));
            Assert.Equal("--- Section 0 ---", SettingsDesk.Row(window, "sec:HEAD0").Label);
            SettingsDesk.Choose(window, "sec:HEAD848");
            Assert.Equal("--- Section 8 ---", window.SettingsShown!.Trail[^1]);
            string last = Parameters(window).Last();
            Assert.Equal("p:P0952", last);
            SettingsDesk.Step(window, last, 1);
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", Huge)["P0952"]);
            Assert.Equal("0.55", SettingsDesk.Value(window, last));
            UiTest.AssertLaidOut(SettingsDesk.Of(window), "shaders-browse-944");
            window.Close();
        }, default);

        // The search over the parameters narrows them by description or id, keeps a heading's rows when the heading matches, and Reset all still reaches every parameter - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public Task The_parameters_are_narrowed_by_a_search_of_their_descriptions_and_ids() => Session.Dispatch(() =>
        {
            MainWindow window = HugeParameters();
            SearchParameters(window, "0917");
            Assert.Equal(["h:HEAD848", "p:P0917"], SettingsDesk.Keys(window).Where(k => k.StartsWith("p:", StringComparison.Ordinal) || k.StartsWith("h:", StringComparison.Ordinal)));
            Assert.Equal("--- Section 8 ---", SettingsDesk.Row(window, "h:HEAD848").Label);
            SettingsDesk.Step(window, "p:P0917", 1);
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", Huge)["P0917"]);

            SearchParameters(window, "p0005");
            Assert.Equal(["p:P0005"], Parameters(window));
            SearchParameters(window, "section 2");
            Assert.Equal(105, Parameters(window).Count());
            Assert.Equal("p:P0213", Parameters(window).First());
            SearchParameters(window, "nothing like it");
            Assert.Empty(Parameters(window));

            // Reset all still reaches every parameter, matched or not.
            SearchParameters(window, "0003");
            SettingsDesk.Choose(window, "paramreset");
            Assert.Empty(GraphicsConfig.Load().ParametersFor("SNES", Huge));
            window.Close();
        }, default);

        // Every row of the big preset's page, and of one heading's page, reached by the pad on the desktop, Down from the first.
        [Fact]
        public Task With_a_long_preset_in_use_every_row_of_its_parameters_is_reached_by_pad() => Session.Dispatch(() =>
        {
            MainWindow window = HugeParameters();
            void EveryRow()
            {
                SettingsLayout layout = SettingsDesk.Layout(window);
                var seen = new HashSet<string?> { layout.Focused };
                for (int i = 0; i <= layout.Rows.Count; i++) { SettingsDesk.Press(window, UiButton.Down); seen.Add(layout.Focused); }
                Assert.Empty(layout.Rows.Where(r => r.IsEnabled && !seen.Contains(r.Key)).Select(r => r.Key));
            }
            EveryRow();
            SettingsDesk.Choose(window, "sec:HEAD0");
            Assert.Equal(105, Parameters(window).Count());
            EveryRow();
            window.Close();
        }, default);

        // The parameters kept are the eight presets read last; a preset that fell out is read again, as the most recent - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public void The_parameter_cache_keeps_the_eight_presets_read_last()
        {
            var shelf = new ShaderShelf(_pack);
            for (int i = 0; i < Row; i++) Assert.Equal(2, shelf.ParametersOf($"slang:row/p{i:D2}.slangp").Count);
            Assert.Equal(Row, shelf.Reads);
            for (int i = 0; i < Row; i++) Assert.Equal(i >= Row - ShaderShelf.CacheLimit, shelf.IsRead($"row/p{i:D2}.slangp"));
            shelf.ParametersOf("slang:row/p11.slangp");
            Assert.Equal(Row, shelf.Reads);
            shelf.ParametersOf("slang:row/p00.slangp");
            Assert.Equal(Row + 1, shelf.Reads);
            Assert.True(shelf.IsRead("row/p00.slangp"));
            Assert.False(shelf.IsRead("row/p04.slangp"));
        }

        [Fact]
        public void A_recent_cache_drops_the_least_recently_used_and_counts_the_unused()
        {
            var cache = new RecentCache<string, int>(3);
            cache.Add("a", 1, used: true);
            cache.Add("b", 2, used: false);
            cache.Add("c", 3, used: false);
            Assert.True(cache.TryGet("b", out int b));
            Assert.Equal(2, b);
            cache.Add("d", 4, used: true);
            Assert.False(cache.Contains("a"));
            Assert.Equal((1, 0), (cache.Evicted, cache.EvictedUnused));
            cache.Add("e", 5, used: true);
            Assert.False(cache.Contains("c"));
            Assert.Equal((2, 1), (cache.Evicted, cache.EvictedUnused));
            Assert.Equal(new[] { true, true, true }, new[] { "b", "d", "e" }.Select(cache.Contains));
            Assert.Throws<ArgumentOutOfRangeException>(() => new RecentCache<string, int>(0));
        }
    }
}
