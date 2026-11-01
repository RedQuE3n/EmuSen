using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Scraping;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Scraper;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit.Abstractions;
using ScriptedScrape = EmuSen.WiseMan.Mistress.BigPicture.WindowFitAuditTests.ScriptedScrape;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The Scraper window: through Mistress's own scraper against the fake server, and through a scripted command for the choice's words, a held store, a run followed and the fit - see EmuSen_LibraryTools.md §9.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScraperWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScraperWindowTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Environment.GetEnvironmentVariable("TMPDIR") is { Length: > 0 } t ? t : Path.GetTempPath(), "EmuSenScraper_" + Guid.NewGuid().ToString("N"));
        private readonly string _romDir;
        private readonly FakeScreenScraper _server = new();
        private readonly Func<string, string> _ask = ScraperPane.Ask;
        private readonly Func<bool> _available = ScraperPane.CommandAvailable;
        private readonly Func<Func<string>, Task<string>> _background = ScraperPane.Background;

        public ScraperWindowTests(ITestOutputHelper output)
        {
            _out = output;
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ScriptedScrape.Restore(_ask, _available, _background);
            NoNetwork.Refuse();
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 30_000) Assert.Fail("timed out waiting for " + what);
                Thread.Sleep(5);
            }
        }


        private ScriptedScrape Script()
        {
            var script = new ScriptedScrape();
            ScraperPane.Ask = script.Ask;
            ScraperPane.CommandAvailable = () => true;
            ScraperPane.Background = work => Task.FromResult(work());
            return script;
        }

        private static ScraperWindow OpenAlone(int width = 1280, int height = 800)
        {
            var window = new ScraperWindow { Width = width, Height = height };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private static List<string> Args(string request) => JsonDocument.Parse(request).RootElement.GetProperty("args").EnumerateArray().Select(a => a.GetString()!).ToList();

        // --- through Mistress's scraper, lent to the command, against the fake server ---

        [Fact]
        public Task Mistress_s_scraper_lists_plans_and_scrapes_the_marked_game_through_the_command() => Session.Dispatch(() =>
        {
            if (!ScrapeCommand.Available) return;
            string rom = Path.Combine(_romDir, "F-Zero (USA).sfc");
            File.WriteAllBytes(rom, SyntheticRom.Build((0x100, [1, 2, 3])));
            _server.Games.Add(new FakeGame(77, "F-Zero", RomHash.Of(rom).Md5));
            NoNetwork.Serve(_server);
            NoNetwork.DeveloperSource.SetValue(null, (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer));
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryGrid, Scraping = true }.Save();
            var main = new MainWindow { Width = 1280, Height = 800 };
            main.Show();
            try
            {
                typeof(MainWindow).GetMethod("ShowScraper", Hidden)!.Invoke(main, null);
                FieldInfo open = typeof(MainWindow).GetField("_scraperWindow", Hidden)!;
                WaitFor(() => open.GetValue(main) is ScraperWindow { IsVisible: true }, "the Scraper shown");
                var scraper = (ScraperWindow)open.GetValue(main)!;
                ScraperPane pane = scraper.Pane;
                WaitFor(() => pane.Rows.Count == 1, "the library's game listed: " + pane.Notice);
                ScrapeRow row = pane.Rows[0];
                Assert.Equal((rom, "snes"), (row.Path, row.System));
                Assert.Equal(["cover", "screenshot", "marquee", "miximage"], row.Media.Select(m => m.Kind));
                Assert.Equal("0 of 4", ScraperPlan.Media(row));
                Assert.True(pane.SetMark(row, ScraperPlan.Scrape.Key));
                Assert.True(pane.OpenPlan());
                Assert.StartsWith("1 game(s), about", pane.PlanText);
                Assert.Empty(_server.JeuInfos);
                Assert.True(pane.Confirm());
                WaitFor(() => { pane.Poll(); return !pane.Following; }, "the run to end");
                Assert.Single(_server.JeuInfos);
                Assert.Equal("Found", pane.OutcomeOf(pane.Rows.Single()));
                // One scraper and one quota: the run the window started is counted by Mistress's own handle.
                Assert.True(main.ScrapeNativeService!.Quota().RequestsToday > 0);
                // While the Scraper is open a run's progress is shown there, not in the status sheet.
                main.ShowScrapeStatus();
                Assert.Null(main.ScrapeStatusShown);
                Assert.Same(scraper, open.GetValue(main));
            }
            finally { main.Close(); }
        }, default);

        // --- the choice is the module's: each section is its words ---

        [Fact]
        public Task Each_section_asks_the_command_for_its_choice_rather_than_filtering_here() => Session.Dispatch(() =>
        {
            ScriptedScrape script = Script();
            ScraperWindow window = OpenAlone();
            ScraperPane pane = window.Pane;
            Assert.Equal(3, pane.Rows.Count);
            Assert.Contains(pane.Sections, s => s.Key == "system:psx");
            foreach ((string section, string[] words) in new[]
            {
                ("favorites", new[] { "ls", "--favorites" }), ("noimage", new[] { "ls", "--missing", ScraperPlan.GameImage }),
                ("nometadata", new[] { "ls", "--missing", "metadata" }), ("missing", new[] { "ls", "--missing", "any" }), ("system:snes", new[] { "ls", "--system", "snes" }),
            })
            {
                // The criteria are the list's filters and the systems its sections (EmuSen_Interface.md §15.1); a system is asked with the filter at All.
                if (section.StartsWith("system:", StringComparison.Ordinal))
                {
                    Assert.True(pane.SelectFilter("all"));
                    Assert.True(pane.SelectSection(section));
                }
                else Assert.True(pane.SelectFilter(section));
                pane.Refresh().GetAwaiter().GetResult();
                Assert.Equal(words, Args(script.Asked.Last(a => Args(a)[0] == "ls")));
            }
            window.Close();
        }, default);

        [Fact]
        public Task The_plan_and_the_run_name_the_marked_games_by_hand_with_the_options() => Session.Dispatch(() =>
        {
            ScriptedScrape script = Script();
            ScraperWindow window = OpenAlone();
            ScraperPane pane = window.Pane;
            window.Pane.Actions.Single(a => a.Text == "Overwrite").Invoke();
            Assert.True(pane.SelectFilter("missing"));
            pane.Refresh().GetAwaiter().GetResult();
            window.Pane.Actions.Single(a => a.Text == "Mark All").Invoke();
            Assert.Equal(2, pane.Marked.Count);
            Assert.True(pane.OpenPlan());
            Assert.Contains("What is there is fetched again and replaced.", pane.PlanText);
            Assert.False(script.Running);
            Assert.True(pane.Confirm());
            Assert.True(script.Running);
            string run = script.Asked.Last(a => Args(a)[0] == "run");
            Assert.Equal(["run", "--overwrite"], Args(run));
            Assert.Equal(ScriptedScrape.Games.Where(g => g.Sources.Contains(null)).Select(g => g.Path).Order(), JsonDocument.Parse(run).RootElement.GetProperty("paths").EnumerateArray().Select(p => p.GetString()!).Order());
            pane.Poll();
            Assert.True(pane.Following);
            Assert.Contains(pane.GetVisualDescendants().OfType<TextBlock>(), t => t.IsVisible && t.Text?.Contains("1 of 2 games") == true);
            script.Running = false;
            pane.Poll();
            Assert.False(pane.Following);
            Assert.StartsWith("2 of 2 game(s) done", pane.Notice);
            window.Close();
        }, default);

        // --- a held store and a run going elsewhere ---

        [Fact]
        public Task A_store_held_by_another_program_is_said_plainly_and_nothing_is_listed() => Session.Dispatch(() =>
        {
            ScriptedScrape script = Script();
            script.Refuse = _ => "the media store is in use by another program (EmuSen or a shell); close it there and try again";
            ScraperWindow window = OpenAlone();
            Assert.Empty(window.Pane.Rows);
            Assert.Equal("The media store is in use by another program (EmuSen or a shell); close it there and try again.", window.Pane.Notice);
            Assert.Contains(window.Pane.Notice!, window.Pane.SummaryText);
            window.Close();
        }, default);

        [Fact]
        public Task A_run_started_elsewhere_is_followed_and_Stop_cancels_it() => Session.Dispatch(() =>
        {
            ScriptedScrape script = Script();
            script.Running = true;
            ScraperWindow window = OpenAlone();
            Assert.True(window.Pane.Following);
            global::EmuSen.LunaP.Commands.LunaAction stop = window.Pane.Actions.Single(a => a.Text == "Stop");
            Assert.True(stop.IsEnabled);
            stop.Invoke();
            Assert.Equal(["cancel"], Args(script.Asked.Last()));
            window.Close();
        }, default);

        // --- fit ---

        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task The_desktop_window_fits_with_nothing_cut_off_or_overlapping(int width, int height) => Session.Dispatch(() =>
        {
            Script();
            ScraperWindow window = OpenAlone(width, height);
            window.Pane.Table.Select(window.Pane.Rows[0]);
            Assert.True(window.Pane.SetMark(window.Pane.Rows[1], ScraperPlan.Scrape.Key));
            for (int i = 0; i < 3; i++) { Dispatcher.UIThread.RunJobs(); UiTest.Capture(window); }
            List<string> faults = FitAudit.Check(window, WindowAllowances.For(window), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge with no fade", StringComparison.Ordinal)).ToList();
            foreach (string f in faults) _out.WriteLine(f);
            Assert.Empty(faults);
            window.Close();
        }, default);
    }
}
