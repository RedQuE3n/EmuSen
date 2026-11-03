using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.Tools.Library;
using EmuSen.Tools.Packages;
using EmuSen.Tools.Scraper;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Each tool's one-line state from its own library's words, a window's listing saying the same as the read without one, and the host keeping and asking them on its schedule - see EmuSen_Interface.md §19.
    [Collection(TestCollections.ProcessGlobals)]
    public class ToolSummaryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ToolSummaryTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenToolSummary_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public ToolSummaryTests()
        {
            _library = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_library);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ToolCounts.Quiet();
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static PackageRow Package(string state, bool packaged) => new() { Key = Guid.NewGuid().ToString(), Title = "g", Console = "SNES", State = state, Package = packaged ? "p" : null };

        private static CoreRow Core(string state, string? trust = null) => new() { Key = Guid.NewGuid().ToString(), Name = "c", Source = CoreRow.Libretro, State = state, Trust = trust };

        [Fact]
        public void Each_tool_words_its_own_line_a_count_needing_the_player_marked_and_nothing_before_a_listing()
        {
            Assert.Null(PackagesWords.StateOf([]));
            Assert.Equal("3 changed", PackagesWords.StateOf([Package("moved", true), Package("missing", true), Package("changed", true), Package("ok", true)])!.Line);
            Assert.Equal("2 packaged", PackagesWords.StateOf([Package("ok", true), Package("ok", true), Package("unpackaged", false)])!.Line);

            ScrapeRow Scrape(params string?[] sources) => new() { Path = "/r/a.sfc", System = "snes", Media = sources.Select(s => ("cover", s)).ToList() };
            Assert.Null(ScraperPlan.StateOf([]));
            ToolSummary art = ScraperPlan.StateOf([Scrape((string?)null), Scrape("ss"), Scrape(null, "ss")])!;
            Assert.Equal(("2 without art", "2 no art", false), (art.Line, art.Brief, art.NeedsAttention));
            Assert.Equal("Every game has art", ScraperPlan.StateOf([Scrape("ss")])!.Line);

            Assert.Null(LibraryFindings.StateOf(complete: false, suspect: 2, duplicates: 1));
            ToolSummary suspect = LibraryFindings.StateOf(true, 2, 1)!;
            Assert.Equal(("⚠ 2 suspect dumps", "⚠ 2 suspect", true), (suspect.Marked, suspect.MarkedBrief, suspect.NeedsAttention));
            Assert.Equal("1 duplicate", LibraryFindings.StateOf(true, 0, 1)!.Marked);

            ToolSummary approve = CoresWords.StateOf([Core("installed", "needs_approval"), Core("update"), Core("available")])!;
            Assert.Equal(("⚠ 1 to approve", true), (approve.Marked, approve.NeedsAttention));
            Assert.Equal("c, from libretro's buildbot, waits for your approval. Nothing downloaded runs until you approve it.", approve.Sentence);
            Assert.Equal("c, a PlayStation 2 core from libretro's buildbot, waits for your approval. Nothing downloaded runs until you approve it.",
                CoresWords.StateOf([Core("installed", "needs_approval") with { ConsoleNames = ["PlayStation 2"] }])!.Sentence);
            Assert.Equal("2 files are suspect dumps: each matches no known dump, or a part of it is missing.", suspect.Sentence);
            Assert.Equal("2 games have no art yet; the Scraper fetches it from ScreenScraper.", art.Sentence);
            Assert.Equal("↑ 2 updates", CoresWords.StateOf([Core("update"), Core("update"), Core("available")])!.Marked);
            Assert.Equal("3 cores · 1 installed", CoresWords.StateOf([Core("installed"), Core("available"), Core("available")])!.Marked);
        }

        [Fact]
        public void Cheats_count_the_codes_switched_on_across_every_saved_list_and_say_nothing_with_none()
        {
            Assert.Null(GameCheats.Summary(CancellationToken.None));
            string snes = Path.Combine(CheatFile.DirectoryPath, "SNES");
            Directory.CreateDirectory(snes);
            CheatFile.SaveTo(Path.Combine(snes, "A.json"), new CheatFile { Cheats = [new() { Description = "a" }, new() { Description = "b" }, new() { Description = "c", Enabled = false }] });
            CheatFile.SaveTo(Path.Combine(snes, "B.json"), new CheatFile { Cheats = [new() { Description = "d" }] });
            CheatFile.SaveTo(Path.Combine(snes, "C.json"), new CheatFile { Cheats = [new() { Description = "e", Enabled = false }] });
            Assert.Equal("3 on · 2 games", GameCheats.Summary(CancellationToken.None)!.Line);
            Assert.Null(GameCheats.Summary(new CancellationToken(canceled: true)));
        }

        [Fact]
        public Task A_packages_listing_says_the_line_its_read_without_a_window_says() => Session.Dispatch(() =>
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            Directory.CreateDirectory(Path.Combine(_library, "snes"));
            File.WriteAllBytes(Path.Combine(_library, "snes", "Cobalt Harbor (Synthetic).sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _library }.Save();
            var window = new PackagesWindow(_library) { Width = 1280, Height = 800 };
            ToolSummary? heard = null;
            window.Pane.SummaryChanged += s => heard = s;
            window.Show();
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (heard is null && clock.Elapsed.TotalSeconds < 60)
            {
                window.Pane.Poll();
                Dispatcher.UIThread.RunJobs();
                Thread.Sleep(5);
            }
            Assert.Equal("None packaged yet", heard?.Line);
            Assert.Equal(heard, PackagesPane.ReadSummary(CancellationToken.None));
            Assert.StartsWith("None packaged yet · 0 bytes in packages", window.Pane.Header!.Facts);
            window.Close();
        }, default);

        private static MainWindow Home(out List<IReadOnlyCollection<string>> asked)
        {
            var calls = new List<IReadOnlyCollection<string>>();
            asked = calls;
            typeof(MainWindow).GetField("ToolSummariesQuery", BindingFlags.Static | BindingFlags.NonPublic)!.SetValue(null,
                (Func<IReadOnlyCollection<string>, CancellationToken, IReadOnlyDictionary<string, ToolSummary?>>)((places, _) =>
                {
                    calls.Add(places.ToList());
                    return places.ToDictionary(p => p, p => (ToolSummary?)ToolCounts.Mockups.GetValueOrDefault(p));
                }));
            typeof(MainWindow).GetField("ToolSummariesBackground", BindingFlags.Static | BindingFlags.NonPublic)!.SetValue(null, (Func<Action, Task>)(work => { work(); return Task.CompletedTask; }));
            new AppSettings { RomDirectory = Path.GetTempPath() + "none" }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private static void Ask(MainWindow w) => typeof(MainWindow).GetMethod("AskToolSummaries", Hidden)!.Invoke(w, []);

        private static ToolSummary? Summary(MainWindow w, string place) => (ToolSummary?)typeof(MainWindow).GetMethod("SummaryOf", Hidden)!.Invoke(w, [place]);

        private static void Age(MainWindow w) => typeof(MainWindow).GetField("_toolSummariesAsked", Hidden)!.SetValue(w, DateTime.MinValue);

        [Fact]
        public Task The_costly_reads_are_made_once_and_again_only_when_their_data_changes_and_a_window_s_listing_is_kept() => Session.Dispatch(() =>
        {
            MainWindow window = Home(out List<IReadOnlyCollection<string>> asked);
            try
            {
                Age(window);
                Ask(window);
                Assert.Equal([HomeScreen.Cores, HomeScreen.Cheats, HomeScreen.Scraper, HomeScreen.Packages], asked.Last());
                asked.Clear();
                Age(window);
                Ask(window);
                Assert.Equal([HomeScreen.Cores, HomeScreen.Cheats], asked.Last());
                Ask(window);
                Assert.Single(asked);

                typeof(MainWindow).GetMethod("ToolDataChanged", Hidden)!.Invoke(window, [HomeScreen.Scraper]);
                Ask(window);
                Assert.Equal([HomeScreen.Cores, HomeScreen.Cheats, HomeScreen.Scraper], asked.Last());

                typeof(MainWindow).GetMethod("ToolSummaryHeard", Hidden)!.Invoke(window, [HomeScreen.Packages, new ToolSummary("1 changed")]);
                Assert.Equal("1 changed", Summary(window, HomeScreen.Packages)?.Line);
                Assert.Equal("1 changed", ((IHomeHost)window).Summary(HomeScreen.Packages)?.Line);
                Assert.Equal("⚠ 1 to approve", Summary(window, HomeScreen.Cores)?.Marked);

                typeof(MainWindow).GetMethod("CancelToolSummaries", Hidden)!.Invoke(window, []);
                Assert.Equal("1 changed", Summary(window, HomeScreen.Packages)?.Line);
            }
            finally { window.Close(); }
        }, default);
    }
}
