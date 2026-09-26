using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Nothing is asked of any server until the player starts a scrape; then ScreenScraper first and OpenEmu's failover where it has nothing - see EmuSen_Settings_Reference.md §4.60.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScrapeWindowTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private static readonly FieldInfo ConfirmField = typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeWindowTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly FakeScrapeClock _clock = new(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero));
        private readonly object _realConfirm = ConfirmField.GetValue(null)!;
        private readonly List<(string Title, string Message)> _asked = new();
        private readonly string _romDir;
        private readonly string _rom;
        private bool _answer = true;

        public ScrapeWindowTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, _clock);
            ConfirmField.SetValue(null, (Func<MainWindow, string, string, Task<bool>>)((_, title, message) =>
            {
                _asked.Add((title, message));
                return Task.FromResult(_answer);
            }));
            _rom = Path.Combine(_romDir, "F-Zero (USA).sfc");
            File.WriteAllBytes(_rom, SyntheticRom.Build((0x100, [1, 2, 3])));
            OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath, ("SNES", "F-Zero (USA)", "00", null), ("SNES", "Other (USA)", "01", null));
            _server.Other = url => url.Contains("thumbnails.libretro.com") ? OnlineCoverTests.FakeServer.Png() : new HttpResponseMessage(HttpStatusCode.NotFound);
        }

        public void Dispose()
        {
            ConfirmField.SetValue(null, _realConfirm);
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Md5 => RomHashes.Of(_rom).Md5;
        private string ScrapedCover => Path.Combine(DataStore.Media, "snes", "covers", "F-Zero (USA).png");
        private string OpenEmuCover => Path.Combine(DataStore.Media, "openemu", "SNES", "F-Zero (USA).png");

        private readonly List<Window> _windows = new();

        // A failed assertion still closes what the test opened, so nothing it started runs on into the next test.
        private Task OnUi(Action body) => Session.Dispatch(() =>
        {
            try { body(); }
            finally { foreach (Window w in _windows) if (w.IsVisible) w.Close(); }
        }, default);

        private static void SetDeveloper(bool present) =>
            NoNetwork.DeveloperSource.SetValue(null, present ? (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer) : NoNetwork.NoDeveloper);

        private MainWindow Open(bool developer = true, Action<AppSettings>? settings = null)
        {
            SetDeveloper(developer);
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryGrid };
            settings?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = 1024, Height = 768 };
            _windows.Add(window);
            window.Show();
            return window;
        }

        private static void WaitFor(Func<bool> condition, string what)
        {
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!condition())
            {
                Dispatcher.UIThread.RunJobs();
                if (clock.ElapsedMilliseconds > 20_000) Assert.Fail("timed out waiting for " + what);
                Thread.Sleep(5);
            }
        }

        // Long enough for the failover's quarter-second spacing and a worker's first request, had either been started.
        private static void Pump(int ms = 800)
        {
            for (int i = 0; i < ms / 10; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }
        }

        private static string? CoverShown(MainWindow w, string rom) =>
            (string?)typeof(MainWindow).GetMethod("CoverPathFor", Hidden)!.Invoke(w, [new RomEntry(rom)]);

        private static void Invoke(MainWindow w, string method) => typeof(MainWindow).GetMethod(method, Hidden, Type.EmptyTypes)!.Invoke(w, null);

        private static void Scrape(MainWindow w, ScrapeScope scope) => Assert.True(w.ConfirmAndScrapeAsync(scope).GetAwaiter().GetResult());

        private static void RunEnds(MainWindow w) => WaitFor(() => w.Progress is { State: not ScrapeRunState.Running }, "the run to end");

        // --- nothing is asked until the player starts it ---

        [Fact]
        public Task Starting_showing_and_refreshing_the_library_asks_no_server() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            Pump();
            Invoke(window, "RefreshLibrary");
            Pump();
            Assert.Empty(_server.Asked);
            Assert.Null(window.ScrapeWorker);
            Assert.Null(CoverShown(window, _rom));
        });

        [Fact]
        public Task A_game_added_to_the_library_asks_no_server() => OnUi(() =>
        {
            MainWindow window = Open();
            File.WriteAllBytes(Path.Combine(_romDir, "Other (USA).sfc"), SyntheticRom.Build((0x100, [9])));
            Invoke(window, "RefreshLibrary");
            Pump();
            Assert.Empty(_server.Asked);
        });

        [Fact]
        public Task The_developer_file_appearing_asks_no_server() => OnUi(() =>
        {
            // media.db already there, as after an earlier session: the store is open when the file appears.
            using (MediaStore.Open(DataStore.Media)) { }
            MainWindow window = Open(developer: false);
            Pump();
            SetDeveloper(true);
            Invoke(window, "ApplyScraping");
            Invoke(window, "RefreshLibrary");
            Pump();
            Assert.Empty(_server.Asked);
            Assert.True(((IScrapeHost)window).HasDeveloperCredentials);
        });

        [Fact]
        public Task A_queue_left_by_an_earlier_session_is_offered_as_resume_and_never_resumed_by_itself() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            using (MediaStore store = MediaStore.Open(DataStore.Media)) store.Enqueue(_rom, "snes", ScrapePriority.Library);
            MainWindow window = Open();
            Pump();
            Assert.Empty(_server.Asked);
            Assert.Equal(1, window.Interrupted);
            Assert.Contains("Resume", ((IScrapeHost)window).Status);

            Assert.True(window.ResumeScrape());
            RunEnds(window);
            Assert.Single(_server.JeuInfos);
            Assert.Equal(ScrapedCover, CoverShown(window, _rom));
        });

        [Fact]
        public Task A_new_run_replaces_what_an_interrupted_one_left_rather_than_adding_to_it() => OnUi(() =>
        {
            string other = Path.Combine(_romDir, "Other (USA).sfc");
            File.WriteAllBytes(other, SyntheticRom.Build((0x100, [9])));
            using (MediaStore store = MediaStore.Open(DataStore.Media)) store.Enqueue(other, "snes", ScrapePriority.Library);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Assert.Equal([Md5], _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")));
            Assert.Equal(1, window.Progress!.Total);
        });

        [Fact]
        public Task What_is_already_kept_is_shown_with_no_request() => OnUi(() =>
        {
            Directory.CreateDirectory(Path.GetDirectoryName(ScrapedCover)!);
            File.WriteAllBytes(ScrapedCover, new byte[200]);
            using (MediaStore store = MediaStore.Open(DataStore.Media))
            {
                var info = new FileInfo(_rom);
                store.RememberFile(_rom, info.Length, info.LastWriteTimeUtc.Ticks, Md5);
                store.Record(new ScrapedRecord(Md5, info.Length, ScrapeState.Found) { Description = "Kept.", FetchedAt = DateTime.UtcNow });
            }
            MainWindow window = Open(developer: false);
            Pump();
            Assert.Equal(ScrapedCover, CoverShown(window, _rom));
            Assert.Empty(_server.Asked);
        });

        [Fact]
        public Task Declining_the_confirm_step_asks_no_server() => OnUi(() =>
        {
            MainWindow window = Open();
            _answer = false;
            Assert.False(window.ConfirmAndScrapeAsync(new ScrapeScope()).GetAwaiter().GetResult());
            Pump();
            Assert.Single(_asked);
            Assert.Empty(_server.Asked);
            Assert.Equal(0, window.Interrupted);
        });

        // --- a run the player started ---

        [Fact]
        public Task Scrape_this_game_takes_screen_scraper_s_cover_and_asks_no_other_server() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Pump();

            Assert.Equal(ScrapedCover, CoverShown(window, _rom));
            Assert.Single(_server.JeuInfos);
            Assert.Empty(_server.OthersAsked);
            Assert.Equal(("Scrape This Game", true), (_asked.Single().Title, _asked.Single().Message.StartsWith("1 game")));
            Assert.Equal((1, 1, ScrapeRunState.Done), (window.Progress!.Total, window.Progress.Found, window.Progress.State));
        });

        [Fact]
        public Task Scrape_this_game_asks_again_what_screen_scraper_once_did_not_know() => OnUi(() =>
        {
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Assert.Equal(2, _server.JeuInfos.Count());
            Assert.Equal(ScrapedCover, CoverShown(window, _rom));
        });

        [Fact]
        public Task Without_the_developer_file_a_run_asks_only_the_failover() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open(developer: false);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Assert.Equal(OpenEmuCover, CoverShown(window, _rom));
            Assert.Empty(_server.Asked.Where(u => u.Contains("screenscraper.fr")));
            Assert.Contains("ScreenScraper cannot be used: EmuSen's developer file is not on this computer", _asked.Single().Message);
            Assert.Contains("OpenEmu's sources are asked", _asked.Single().Message);
        });

        [Fact]
        public Task A_run_without_screen_scraper_asks_the_failover_only_for_games_with_no_cover() => OnUi(() =>
        {
            string hand = Path.Combine(DataStore.Artwork, "SNES", "Other (USA).png");
            Directory.CreateDirectory(Path.GetDirectoryName(hand)!);
            File.WriteAllBytes(hand, new byte[200]);
            File.WriteAllBytes(Path.Combine(_romDir, "Other (USA).sfc"), SyntheticRom.Build((0x100, [9])));
            MainWindow window = Open(developer: false);
            WaitFor(() => CoverShown(window, Path.Combine(_romDir, "Other (USA).sfc")) == hand, "the player's cover");
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Pump();
            Assert.Equal(2, window.Progress!.Total);
            Assert.Equal(["F-Zero%20%28USA%29.png"], _server.OthersAsked.Select(u => u[(u.LastIndexOf('/') + 1)..]));
        });

        [Fact]
        public Task A_game_screen_scraper_does_not_know_goes_to_the_failover_in_the_same_run() => OnUi(() =>
        {
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Assert.Equal(OpenEmuCover, CoverShown(window, _rom));
            Assert.Single(_server.JeuInfos);
            Assert.Equal(1, window.Progress!.FailoverFound);
        });

        [Fact]
        public Task A_game_screen_scraper_knows_without_a_box_goes_to_the_failover_and_keeps_screen_scraper_s_other_media() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5) { Media = [("ss", "wor", "png")] });
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Assert.Equal(OpenEmuCover, CoverShown(window, _rom));
            Assert.True(File.Exists(Path.Combine(DataStore.Media, "snes", "screenshots", "F-Zero (USA).png")));
        });

        [Theory]
        [InlineData(430)]
        [InlineData(431)]
        [InlineData(423)]
        [InlineData(426)]
        [InlineData(403)]
        public Task When_screen_scraper_stops_the_failover_takes_what_is_left_and_the_run_says_why(int code) => OnUi(() =>
        {
            _server.ForcedStatus = code;
            MainWindow window = Open();
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Assert.Equal(OpenEmuCover, CoverShown(window, _rom));
            Assert.Single(_server.JeuInfos);
            Assert.Equal(ScrapeRunState.Stopped, window.Progress!.State);
            Assert.Contains(code.ToString(), window.FindControl<TextBlock>("StatusText")!.Text);
            Assert.Equal(1, window.Interrupted);
        });

        [Fact]
        public Task A_day_already_used_up_goes_straight_to_the_failover_and_the_plan_says_so() => OnUi(() =>
        {
            using (MediaStore store = MediaStore.Open(DataStore.Media))
                store.SaveDay(new QuotaDay(ScrapeQuotaManager.DayOf(_clock.Now), 20000, 3, 20000, 2000, ScrapeQuotaManager.StartOfNextDay(_clock.Now), "used up"));
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Assert.Contains("cannot be used: used up", _asked.Single().Message);
            Assert.Equal(OpenEmuCover, CoverShown(window, _rom));
            Assert.Empty(_server.JeuInfos);
        });

        [Fact]
        public Task With_the_failover_off_a_run_asks_no_other_server_and_its_old_covers_are_not_shown() => OnUi(() =>
        {
            Directory.CreateDirectory(Path.GetDirectoryName(OpenEmuCover)!);
            File.WriteAllBytes(OpenEmuCover, new byte[200]);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, ScrapeScope.ThisGame(_rom));
            RunEnds(window);
            Pump();
            Assert.Single(_server.JeuInfos);
            Assert.Empty(_server.OthersAsked);
            Assert.Null(CoverShown(window, _rom));
        });

        [Fact]
        public Task A_cover_the_player_placed_wins_and_missing_art_leaves_that_game_out() => OnUi(() =>
        {
            string hand = Path.Combine(DataStore.Artwork, "SNES", "F-Zero (USA).png");
            Directory.CreateDirectory(Path.GetDirectoryName(hand)!);
            File.WriteAllBytes(hand, new byte[200]);
            Directory.CreateDirectory(Path.GetDirectoryName(ScrapedCover)!);
            File.WriteAllBytes(ScrapedCover, new byte[200]);
            File.WriteAllBytes(Path.Combine(_romDir, "Other (USA).sfc"), SyntheticRom.Build((0x100, [9])));
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == hand, "the player's cover");
            Assert.Equal(1, window.Plan(new ScrapeScope(MissingArtOnly: true)).Games);
            Assert.Equal(2, window.Plan(new ScrapeScope()).Games);
        });

        [Fact]
        public Task A_console_scope_names_its_shelf_game_boy_color_on_its_own() => OnUi(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Zelda (USA).nes"), new byte[64]);
            byte[] gbc = new byte[0x8000];
            gbc[0x143] = 0xC0;
            File.WriteAllBytes(Path.Combine(_romDir, "Colour (USA).gbc"), gbc);
            File.WriteAllBytes(Path.Combine(_romDir, "Mono (USA).gb"), new byte[0x8000]);
            MainWindow window = Open();
            Assert.Equal(1, window.Plan(new ScrapeScope(Shelf: EmuSen.Cores.CoreCatalog.GameBoyColorShelf)).Games);
            Assert.Equal(1, window.Plan(new ScrapeScope(Shelf: "NES (Moon)")).Games);
            Assert.Equal(4, window.Plan(new ScrapeScope()).Games);

            _server.Gate = new SemaphoreSlim(0);
            Scrape(window, new ScrapeScope(Shelf: EmuSen.Cores.CoreCatalog.GameBoyColorShelf));
            Assert.Equal(["gbc"], window.ScrapeStore!.Queue().Select(q => q.System));
            _server.Gate.Release(100);
            RunEnds(window);
        });

        [Fact]
        public Task The_plan_counts_the_games_and_what_they_will_cost_before_anything_starts() => OnUi(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Other (USA).sfc"), SyntheticRom.Build((0x100, [9])));
            MainWindow window = Open();
            ScrapePlan plan = window.Plan(new ScrapeScope());
            Assert.Equal((2, 2, 10, true), (plan.Games, plan.NotYetAsked, plan.MaxRequests, plan.ScreenScraperUsable));
            Assert.Contains("up to 10 requests", plan.Describe());
            Assert.Empty(_server.Asked);
        });

        [Fact]
        public Task A_run_shows_its_progress_and_a_cancelled_run_asks_nothing_more_and_resumes_from_where_it_was() => OnUi(() =>
        {
            for (int i = 0; i < 5; i++) File.WriteAllBytes(Path.Combine(_romDir, $"Game {i} (USA).sfc"), SyntheticRom.Build((0x100, [(byte)(10 + i)])));
            _server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            Scrape(window, new ScrapeScope());
            _server.Gate.Release(1);
            WaitFor(() => window.Progress!.Done == 1, "one game");
            Assert.Contains("Scraping: 1 of 6", ((IScrapeHost)window).Status);

            window.CancelScrape();
            int asked = _server.Asked.Count;
            _server.Gate.Release(100);
            Pump();
            Assert.Equal(asked, _server.Asked.Count);
            Assert.Equal(ScrapeRunState.Cancelled, window.Progress!.State);
            Assert.Equal(5, window.Interrupted);

            Assert.True(window.ResumeScrape());
            RunEnds(window);
            Assert.Equal(5, window.Progress!.Total);
            Assert.Equal(6, _server.JeuInfos.Select(u => FakeScreenScraper.Param(u, "md5")).Distinct().Count());
        });

        // Without StopScraping in Closing the worker goes on asking after the window is gone; this fails then.
        [Fact]
        public Task A_closed_window_asks_nothing_more_and_closes_its_worker_and_its_store() => OnUi(() =>
        {
            for (int i = 0; i < 6; i++) File.WriteAllBytes(Path.Combine(_romDir, $"Game {i} (USA).sfc"), SyntheticRom.Build((0x100, [(byte)(10 + i)])));
            _server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open();
            Scrape(window, new ScrapeScope());
            WaitFor(() => _server.JeuInfos.Any(), "the first request");
            Scraper worker = window.ScrapeWorker!;
            MediaStore store = window.ScrapeStore!;

            window.Close();
            int asked = _server.Asked.Count;
            _server.Gate.Release(100);
            Pump();

            Assert.Equal(asked, _server.Asked.Count);
            Assert.False(worker.IsRunning);
            Assert.False(store.IsOpen);
            Assert.Null(window.ScrapeWorker);
        });

        // --- where the player starts it ---

        [Fact]
        public Task The_library_s_menu_and_the_pad_menu_offer_scrape_this_game_and_the_pad_menu_opens_the_scraping_tab() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            Pump(100);
            var entries = (List<PadMenuEntry>)typeof(MainWindow).GetField("_padMenuEntries", Hidden)!.GetValue(window)!;
            window.GetControl<ListBox>("LibraryList").SelectedIndex = -1;
            Invoke(window, "OpenPadMenu");
            Assert.DoesNotContain(entries, e => e.Text() == "Scrape This Game...");
            Invoke(window, "ClosePadMenu");

            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "OpenPadMenu");
            entries.Single(e => e.Text() == "Scrape This Game...").Accept();
            RunEnds(window);
            Assert.Equal("Scrape This Game", _asked.Single().Title);
            Assert.Single(_server.JeuInfos);

            Invoke(window, "OpenPadMenu");
            entries.Single(e => e.Text() == "Scrape Games...").Accept();
            Pump(100);
            PreferencesWindow prefs = window.OwnedWindows.OfType<PreferencesWindow>().Single();
            TabControl tabs = prefs.GetVisualDescendants().OfType<TabControl>().Single(t => t.Name == "PreferenceTabs");
            Assert.Equal(PreferencesWindow.ScrapingTab, (string?)((TabItem)tabs.SelectedItem!).Header);
            Assert.Single(_server.JeuInfos);
            prefs.Close();
        });

        [Fact]
        public Task The_scraping_tab_s_button_asks_first_then_starts_and_lets_go_of_the_window_when_it_closes() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            var prefs = new PreferencesWindow(new AppSettings { RomDirectory = _romDir }, window);
            _windows.Add(prefs);
            prefs.Show();
            prefs.ShowTab(PreferencesWindow.ScrapingTab);
            Pump(100);
            Assert.NotNull(ScrapeChangedHandlers(window));
            Assert.Empty(_server.Asked);

            Button start = prefs.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "ScrapeStartButton");
            start.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            RunEnds(window);
            Assert.Single(_asked);
            Assert.Equal(ScrapedCover, CoverShown(window, _rom));

            prefs.Close();
            Pump(100);
            Assert.DoesNotContain(ScrapeChangedHandlers(window)?.GetInvocationList() ?? [], d => d.Target is ScrapePreferencesPane);
        });

        // The member account is kept only through Log In: ScrapeSignInTests.

        private static Delegate? ScrapeChangedHandlers(MainWindow w) =>
            (Delegate?)typeof(MainWindow).GetField("ScrapeChanged", Hidden)!.GetValue(w);

        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public void An_old_online_covers_setting_leaves_the_failover_on_and_is_dropped_at_the_next_save(bool old)
        {
            Directory.CreateDirectory(ConfigStore.Directory);
            File.WriteAllText(ConfigStore.For("appsettings.json"), $"{{ \"OnlineCovers\": {(old ? "true" : "false")} }}");
            AppSettings settings = AppSettings.Load();
            Assert.True(settings.OpenEmuFallback);
            Assert.Null(settings.OnlineCovers);
            settings.Save();
            Assert.DoesNotContain("OnlineCovers", File.ReadAllText(ConfigStore.For("appsettings.json")));
        }
    }
}
