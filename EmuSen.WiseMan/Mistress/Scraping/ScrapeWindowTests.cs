using System;
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
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // The window: ScreenScraper first, OpenEmu's failover where it has nothing or cannot be used, and everything closed with the window - see EmuSen_Settings_Reference.md §4.60.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScrapeWindowTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScrapeWindowTests", Guid.NewGuid().ToString("N"));
        private readonly FakeScreenScraper _server = new();
        private readonly FakeScrapeClock _clock = new(new DateTimeOffset(2026, 7, 14, 10, 0, 0, TimeSpan.Zero));
        private readonly string _romDir;
        private readonly string _rom;

        public ScrapeWindowTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            NoNetwork.HttpFactory.SetValue(null, (Func<HttpClient>)(() => new HttpClient(_server)));
            NoNetwork.ScrapeClock.SetValue(null, _clock);
            _rom = Path.Combine(_romDir, "F-Zero (USA).sfc");
            File.WriteAllBytes(_rom, SyntheticRom.Build((0x100, [1, 2, 3])));
            OnlineCoverTests.BuildOpenVgdb(OpenVgdb.DefaultPath, ("SNES", "F-Zero (USA)", "00", null));
            _server.Other = url => url.Contains("thumbnails.libretro.com") ? OnlineCoverTests.FakeServer.Png() : new HttpResponseMessage(HttpStatusCode.NotFound);
        }

        public void Dispose()
        {
            NoNetwork.Refuse();
            NoNetwork.ScrapeClock.SetValue(null, SystemScrapeClock.Instance);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch { }
        }

        private string Md5 => RomHashes.Of(_rom).Md5;
        private string ScrapedCover => Path.Combine(DataStore.Media, "snes", "covers", "F-Zero (USA).png");
        private string OpenEmuCover => Path.Combine(DataStore.Media, "openemu", "SNES", "F-Zero (USA).png");

        private readonly System.Collections.Generic.List<Window> _windows = new();

        // A failed assertion still closes what the test opened, so nothing it started runs on into the next test.
        private Task OnUi(Action body) => Session.Dispatch(() =>
        {
            try { body(); }
            finally { foreach (Window w in _windows) if (w.IsVisible) w.Close(); }
        }, default);

        private MainWindow Open(bool developer = true, Action<AppSettings>? settings = null)
        {
            NoNetwork.DeveloperSource.SetValue(null, developer ? (Func<DeveloperCredentials?>)(() => FakeScreenScraper.Developer) : NoNetwork.NoDeveloper);
            var app = new AppSettings { RomDirectory = _romDir };
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

        private static void Pump(int ms = 400)
        {
            for (int i = 0; i < ms / 10; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(10); }
        }

        private static string? CoverShown(MainWindow w, string rom) =>
            (string?)typeof(MainWindow).GetMethod("CoverPathFor", Hidden)!.Invoke(w, [new RomEntry(rom)]);

        [Fact]
        public Task With_the_developer_file_a_shown_game_s_cover_comes_from_screen_scraper_and_no_other_server_is_asked() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == ScrapedCover, "ScreenScraper's cover");

            Assert.Single(_server.JeuInfos);
            Assert.Empty(_server.OthersAsked);
            Assert.False(File.Exists(OpenEmuCover));
            window.Close();
        });

        [Fact]
        public Task Without_the_developer_file_screen_scraper_is_never_asked_and_open_emu_s_failover_fills_the_cover() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open(developer: false);
            WaitFor(() => CoverShown(window, _rom) == OpenEmuCover, "the failover's cover");

            Assert.Empty(_server.Asked.Where(u => u.Contains("screenscraper.fr")));
            Assert.Contains("thumbnails.libretro.com", _server.OthersAsked.Single());
            Assert.False(File.Exists(Path.Combine(DataStore.Artwork, "SNES", "F-Zero (USA).png")));
            window.Close();
        });

        [Fact]
        public Task A_game_screen_scraper_does_not_know_goes_to_the_failover() => OnUi(() =>
        {
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == OpenEmuCover, "the failover's cover");
            Assert.Single(_server.JeuInfos);
            window.Close();
        });

        [Fact]
        public Task A_game_screen_scraper_knows_without_a_box_goes_to_the_failover_and_keeps_screen_scraper_s_other_media() => OnUi(() =>
        {
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5) { Media = [("ss", "wor", "png")] });
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == OpenEmuCover, "the failover's cover");
            Assert.True(File.Exists(Path.Combine(DataStore.Media, "snes", "screenshots", "F-Zero (USA).png")));
            window.Close();
        });

        [Theory]
        [InlineData(430)]
        [InlineData(431)]
        [InlineData(423)]
        [InlineData(426)]
        [InlineData(403)]
        public Task When_screen_scraper_stops_the_failover_takes_the_covers_and_the_status_line_says_why(int code) => OnUi(() =>
        {
            _server.ForcedStatus = code;
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == OpenEmuCover, "the failover's cover");
            Assert.Single(_server.JeuInfos);
            Assert.Contains(code.ToString(), window.FindControl<TextBlock>("StatusText")!.Text);
            window.Close();
        });

        [Fact]
        public Task A_day_already_used_up_goes_straight_to_the_failover_without_asking_screen_scraper() => OnUi(() =>
        {
            using (MediaStore store = MediaStore.Open(DataStore.Media))
                store.SaveDay(new QuotaDay(ScrapeQuotaManager.DayOf(_clock.Now), 20000, 3, 20000, 2000, ScrapeQuotaManager.StartOfNextDay(_clock.Now), "used up"));
            _server.Games.Add(new FakeGame(77, "F-Zero", Md5));
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == OpenEmuCover, "the failover's cover");
            Assert.Empty(_server.JeuInfos);
            window.Close();
        });

        [Fact]
        public Task With_the_failover_off_a_game_screen_scraper_does_not_know_asks_no_other_server() => OnUi(() =>
        {
            MainWindow window = Open(settings: a => a.OpenEmuFallback = false);
            WaitFor(() => _server.JeuInfos.Any(), "ScreenScraper's answer");
            Pump();
            Assert.Empty(_server.OthersAsked);
            Assert.Null(CoverShown(window, _rom));
            window.Close();

            window = Open(developer: false, settings: a => a.OpenEmuFallback = false);
            Pump();
            Assert.Empty(_server.OthersAsked);
            window.Close();
        });

        [Fact]
        public Task A_cover_the_player_placed_wins_over_screen_scraper_s() => OnUi(() =>
        {
            string hand = Path.Combine(DataStore.Artwork, "SNES", "F-Zero (USA).png");
            Directory.CreateDirectory(Path.GetDirectoryName(hand)!);
            File.WriteAllBytes(hand, new byte[200]);
            Directory.CreateDirectory(Path.GetDirectoryName(ScrapedCover)!);
            File.WriteAllBytes(ScrapedCover, new byte[200]);
            MainWindow window = Open();
            WaitFor(() => CoverShown(window, _rom) == hand, "the player's cover");
            Pump();
            Assert.Equal(hand, CoverShown(window, _rom));
            Assert.Empty(_server.OthersAsked);
            window.Close();
        });

        // Without StopScraping in Closing the worker goes on asking after the window is gone; this fails then.
        [Fact]
        public Task A_closed_window_asks_nothing_more_and_closes_its_worker_and_its_store() => OnUi(() =>
        {
            for (int i = 0; i < 6; i++) File.WriteAllBytes(Path.Combine(_romDir, $"Game {i} (USA).sfc"), SyntheticRom.Build((0x100, [(byte)(10 + i)])));
            _server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open();
            WaitFor(() => _server.JeuInfos.Any(), "the first request");
            Scraper worker = (Scraper)typeof(MainWindow).GetProperty("ScrapeWorker", Hidden)!.GetValue(window)!;
            MediaStore store = (MediaStore)typeof(MainWindow).GetProperty("ScrapeStore", Hidden)!.GetValue(window)!;

            window.Close();
            int asked = _server.Asked.Count;
            _server.Gate.Release(100);
            Pump(800);

            Assert.Equal(asked, _server.Asked.Count);
            Assert.False(worker.IsRunning);
            Assert.False(store.IsOpen);
            Assert.Null(typeof(MainWindow).GetProperty("ScrapeWorker", Hidden)!.GetValue(window));
        });

        [Fact]
        public Task The_whole_library_is_queued_the_console_in_view_first() => OnUi(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Zelda (USA).nes"), new byte[64]);
            File.WriteAllBytes(Path.Combine(_romDir, "Tetris (World).gb"), new byte[512]);
            _server.Gate = new SemaphoreSlim(0);
            MainWindow window = Open(settings: a => a.SelectedCore = "NES (Moon)");
            window.ScrapeWholeLibrary();
            MediaStore store = (MediaStore)typeof(MainWindow).GetProperty("ScrapeStore", Hidden)!.GetValue(window)!;
            var queue = store.Queue().Select(q => (q.System, q.Priority)).ToList();
            Assert.Equal(["nes", "snes", "gb"], queue.Select(q => q.System).Distinct());
            Assert.All(queue.Where(q => q.System == "nes"), q => Assert.True(q.Priority <= ScrapePriority.Console));
            Assert.All(queue.Where(q => q.System != "nes"), q => Assert.Equal(ScrapePriority.Library, q.Priority));
            window.Close();
        });

        [Fact]
        public Task Preferences_keeps_the_member_account_in_its_own_file_and_lets_go_of_the_window_when_it_closes() => OnUi(() =>
        {
            MainWindow window = Open();
            var prefs = new PreferencesWindow(new AppSettings { RomDirectory = _romDir }, window);
            _windows.Add(prefs);
            prefs.Show();
            TabControl tabs = prefs.GetVisualDescendants().OfType<TabControl>().Single(t => t.Name == "PreferenceTabs");
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().Single(t => (string?)t.Header == "Scraping");
            Pump(100);
            prefs.UpdateLayout();
            Assert.NotNull(ScrapeChangedHandlers(window));

            prefs.GetVisualDescendants().OfType<TextBox>().Single(t => t.Name == "ScreenScraperUserBox").Text = "FAKEMEMBERUSER";
            prefs.GetVisualDescendants().OfType<TextBox>().Single(t => t.Name == "ScreenScraperPasswordBox").Text = "FAKEMEMBERSECRET";
            Assert.False(File.Exists(MemberAccount.PathOf));
            prefs.Close();
            Pump(100);

            Assert.Equal(("FAKEMEMBERUSER", "FAKEMEMBERSECRET"), (MemberAccount.Load().User, MemberAccount.Load().Password));
            Assert.Equal(UnixFileMode.UserRead | UnixFileMode.UserWrite, File.GetUnixFileMode(MemberAccount.PathOf));
            Assert.DoesNotContain("FAKEMEMBER", File.ReadAllText(ConfigStore.For("appsettings.json")));
            Assert.Null(ScrapeChangedHandlers(window));
            window.Close();
        });

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
