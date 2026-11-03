using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Headless;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Mistress.Scraping
{
    // Pass 8 in the window: Find by Name and its chooser, the criteria, the plan's prices, Refresh, ScreenScraper's names, and the cleanup of orphaned media - see EmuSen_BigPicture.md §38.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScrapeExtrasWindowTests : ScrapeWindowFixture, IDisposable
    {
        private static readonly FieldInfo CleanUpField = typeof(MainWindow).GetField("ConfirmCleanUp", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realCleanUp = CleanUpField.GetValue(null)!;
        private readonly List<string> _cleanUpAsked = new();
        private bool _cleanUpAnswer = true;

        public ScrapeExtrasWindowTests(ITestOutputHelper output) : base(output)
        {
            CleanUpField.SetValue(null, (Func<MainWindow, string, Task<bool>>)((_, message) => { _cleanUpAsked.Add(message); return Task.FromResult(_cleanUpAnswer); }));
        }

        public new void Dispose()
        {
            CleanUpField.SetValue(null, _realCleanUp);
            base.Dispose();
        }

        private static AppSettings SettingsOf(MainWindow w) => (AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(w)!;

        private static GameRecords RecordsOf(MainWindow w) => (GameRecords)typeof(MainWindow).GetField("_records", Hidden)!.GetValue(w)!;

        private static string? CoverShown(MainWindow w, string rom) =>
            (string?)typeof(MainWindow).GetMethod("CoverPathFor", Hidden)!.Invoke(w, [new RomEntry(rom)]);

        private static void Refresh(MainWindow w)
        {
            typeof(MainWindow).GetMethod("RefreshLibrary", Hidden, Type.EmptyTypes)!.Invoke(w, null);
            Pump(200);
        }

        private string Stored(params string[] parts) => Path.Combine([DataStore.Media, .. parts]);

        private static Dictionary<string, (long, DateTime)> Fingerprint(string dir) =>
            Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories).ToDictionary(f => f, f => (new FileInfo(f).Length, File.GetLastWriteTimeUtc(f)));

        // --- Find by Name ---

        [Fact]
        public Task Each_search_waits_its_turn_at_the_quota_s_pace() => OnUi(() =>
        {
            string rom = Game("Nameless", 9);
            Server.User = FakeScreenScraper.Quota(maxThreads: 1, perMinute: 6, perDay: 20000, koPerDay: 2000, today: 10, koToday: 1);
            MainWindow window = Open();
            window.Plan(ScrapeScope.ThisGame(rom));
            DateTimeOffset start = window.ScrapeNow;
            for (int i = 0; i < 3; i++) window.SearchByNameAsync(rom, "Nothing " + i).GetAwaiter().GetResult();
            // The first answer sets 6 a minute less 10%; the third waits that interval after the second.
            Assert.Equal(3, Server.Searches.Count());
            Assert.True(window.ScrapeNow - start >= TimeSpan.FromSeconds(60 / 5.4), $"{(window.ScrapeNow - start).TotalSeconds} s");
        });

        // --- the criteria and the plan ---

        [Fact]
        public Task Each_criterion_chooses_the_games_a_wide_run_takes() => OnUi(() =>
        {
            string scraped = Game("Scraped (USA)", 1), favourite = Game("Favourite (USA)", 3), shot = Game("Shot Only (USA)", 5), clip = Game("Clip Only (USA)", 7), bare = Game("Bare (USA)", 9);
            string described = Game("Described (USA)", 11);
            Server.Games.Add(new FakeGame(1, "Scraped", Md5(scraped)));
            MainWindow window = Open();
            Pump(200);
            Scrape(window, ScrapeScope.ThisGame(scraped));
            RunEnds(window);
            Pump(200);
            RecordsOf(window).ToggleFavourite(favourite);
            RecordsOf(window).SaveEdits(described, new Dictionary<string, string?> { [GameMetadata.Description] = "Typed by the player." }, DateTime.Now);
            Directory.CreateDirectory(Stored("snes", "screenshots"));
            File.WriteAllBytes(Stored("snes", "screenshots", "Shot Only (USA).png"), new byte[100]);
            Directory.CreateDirectory(Stored("snes", "videos"));
            File.WriteAllBytes(Stored("snes", "videos", "Clip Only (USA).mp4"), new byte[100]);
            Refresh(window);

            int Count(ScrapeCriteria c) => window.Plan(new ScrapeScope { Criteria = c }).Games;
            Assert.Equal(6, Count(ScrapeCriteria.All));
            Assert.Equal(1, Count(ScrapeCriteria.Favourites));
            Assert.Equal(4, Count(ScrapeCriteria.NoMetadata));
            Assert.Equal(5, Count(ScrapeCriteria.NoCover));
            Assert.Equal(4, Count(ScrapeCriteria.NoGameImage));
            Assert.Equal(5, Count(ScrapeCriteria.NoGameVideo));
            Assert.Equal(5, window.Plan(new ScrapeScope(MissingArtOnly: true)).Games);
            // One game is still one game, whatever the criteria say.
            Assert.Equal(1, window.Plan(ScrapeScope.ThisGame(scraped) with { Criteria = ScrapeCriteria.NoCover }).Games);
            Assert.Equal(1, Server.JeuInfos.Count());
            _ = bare;
        });

        [Fact]
        public Task The_confirm_step_prices_each_kind_turned_on_by_its_offer_rate_and_a_refresh_by_its_lookups() => OnUi(() =>
        {
            string a = Game("Alpha (USA)", 1), b = Game("Beta (USA)", 3);
            Server.Games.Add(new FakeGame(1, "Alpha", Md5(a)));
            Server.Games.Add(new FakeGame(2, "Beta", Md5(b)));
            MainWindow window = Open(settings: s => s.ScrapeBackCovers = true);
            Pump(200);

            ScrapePlan plan = window.Plan(new ScrapeScope());
            Assert.Equal((2, 2, 12), (plan.Games, plan.NotYetAsked, plan.MaxRequests));
            double perGame = 1 + (33 + 36 + 36 + 39 + 33) / 39.0;
            Assert.Equal(2 * perGame, plan.ExpectedRequests, 3);
            Assert.Equal(2 * (33 * 653 + 36 * 5 + 36 * 90 + 39 * 565 + 33 * 468) / 39.0 / 1024, plan.ExpectedMB, 3);
            Assert.Contains($"about {Math.Round(2 * perGame):N0} and", plan.Describe());

            Scrape(window, new ScrapeScope());
            RunEnds(window);
            Pump(200);
            int before = Server.Asked.Count;
            Assert.Equal(0, window.Plan(new ScrapeScope()).AskedAgain);
            ScrapePlan refresh = window.Plan(new ScrapeScope { Refresh = true });
            Assert.Equal((2, 0, 2), (refresh.Games, refresh.NotYetAsked, refresh.AskedAgain));
            Assert.Equal(2, refresh.ExpectedRequests, 3);
            Assert.Contains("asked again to refresh them", refresh.Describe());

            Scrape(window, new ScrapeScope { Refresh = true });
            RunEnds(window);
            Assert.Equal(before + 2, Server.Asked.Count);
            Assert.Equal(8, window.Progress!.Unchanged);
        });

        [Fact]
        public Task Settings_scraping_offers_each_new_kind_off_and_refresh_and_saves_them() => OnUi(() =>
        {
            MainWindow window = Open();
            SettingsDesk.At(window, SettingsScreen.Library, "scraping");
            Pump(100);

            string[] kinds = ["backs", "boxes3d", "physical", "fanart", "manuals", "videos", "gamenames", "refresh"];
            foreach (string key in kinds) Assert.Equal(SettingsScreen.Off, SettingsDesk.Value(window, key));

            foreach (string key in kinds) SettingsDesk.Pick(window, key, SettingsScreen.On);
            Pump(100);
            AppSettings saved = AppSettings.Load();
            Assert.True(saved.ScrapeBackCovers && saved.Scrape3DBoxes && saved.ScrapePhysicalMedia && saved.ScrapeFanArt && saved.ScrapeManuals && saved.ScrapeVideos && saved.ScrapeGameNames && saved.ScrapeRefresh);
            Assert.Empty(Server.Asked);
        });

        [Fact]
        public Task Settings_Refresh_reaches_Scrape_This_Game() => OnUi(() =>
        {
            string liked = Game("Liked (USA)", 1);
            Server.Games.Add(new FakeGame(1, "Liked", Md5(liked)));
            MainWindow window = Open();
            Pump(200);
            SettingsDesk.At(window, SettingsScreen.Library, "scraping");
            Pump(100);
            Scrape(window, ScrapeScope.ThisGame(liked));
            RunEnds(window);
            Assert.Equal(Md5(liked), FakeScreenScraper.Param(Server.JeuInfos.Single(), "md5"));

            // Scrape This Game follows "Refresh what is kept", as ES-DE's single-game scraper follows its overwrite setting.
            int before = Server.JeuInfos.Count();
            Scrape(window, ScrapeScope.ThisGame(liked));
            RunEnds(window);
            Assert.Equal(before, Server.JeuInfos.Count());
            SettingsDesk.Pick(window, "refresh", SettingsScreen.On);
            Scrape(window, ScrapeScope.ThisGame(liked));
            RunEnds(window);
            Assert.Equal(before + 1, Server.JeuInfos.Count());
            Assert.Equal(4, window.Progress!.Unchanged);
        });


        // --- ScreenScraper's names (Q30) ---

        // --- orphaned media ---

        [Fact]
        public Task Clean_up_asks_first_then_moves_only_the_media_of_games_gone_and_never_touches_the_rom_folder() => OnUi(() =>
        {
            string kept = Game("Kept (USA)", 1), gone = Game("Gone (USA)", 3);
            Server.Games.Add(new FakeGame(1, "Kept", Md5(kept)));
            Server.Games.Add(new FakeGame(2, "Gone", Md5(gone)));
            MainWindow window = Open();
            Pump(200);
            Scrape(window, new ScrapeScope());
            RunEnds(window);
            // A console with no game in the library is left alone, as ES-DE cleans only systems it has.
            Directory.CreateDirectory(Stored("nes", "covers"));
            File.WriteAllBytes(Stored("nes", "covers", "Elsewhere.png"), new byte[100]);
            (string goneMd5, long goneBytes) = (Md5(gone), new FileInfo(gone).Length);
            Assert.NotEmpty(window.ScrapeMedia(goneMd5, goneBytes));
            Directory.CreateDirectory(Stored("snes", "manuals"));
            File.WriteAllBytes(Stored("snes", "manuals", "Gone (USA).pdf"), System.Text.Encoding.ASCII.GetBytes("%PDF-1.4 a manual"));
            File.Delete(gone);
            Refresh(window);
            var roms = Fingerprint(RomDir);
            int requests = Server.Asked.Count;

            IReadOnlyList<string> orphans = window.OrphanedMedia();
            Assert.Equal(5, orphans.Count);
            Assert.All(orphans, o => Assert.Contains("Gone (USA)", o));

            _cleanUpAnswer = false;
            Assert.Equal("Nothing was moved.", window.CleanUpOrphansAsync().GetAwaiter().GetResult());
            Assert.StartsWith("5 scraped file(s)", Assert.Single(_cleanUpAsked));
            Assert.True(File.Exists(Stored("snes", "covers", "Gone (USA).png")));

            _cleanUpAnswer = true;
            string said = window.CleanUpOrphansAsync().GetAwaiter().GetResult();
            Assert.StartsWith("Moved 5 file(s)", said);
            string backup = Assert.Single(Directory.GetDirectories(Stored(MediaStore.CleanupFolder)));
            Assert.True(File.Exists(Path.Combine(backup, "snes", "covers", "Gone (USA).png")));
            Assert.True(File.Exists(Path.Combine(backup, "cleanup.txt")));
            // A folder the cleanup emptied goes, as ES-DE removes empty media folders; one still holding a kept game's file stays.
            Assert.False(Directory.Exists(Stored("snes", "manuals")));
            Assert.True(Directory.Exists(Stored("snes", "covers")));
            Assert.True(File.Exists(Path.Combine(backup, "snes", "manuals", "Gone (USA).pdf")));
            Assert.False(File.Exists(Stored("snes", "covers", "Gone (USA).png")));
            Assert.True(File.Exists(Stored("snes", "covers", "Kept (USA).png")));
            Assert.True(File.Exists(Stored("nes", "covers", "Elsewhere.png")));
            Assert.Empty(window.ScrapeMedia(goneMd5, goneBytes));
            Assert.Empty(window.OrphanedMedia());
            Assert.Equal(roms, Fingerprint(RomDir));
            Assert.Equal(requests, Server.Asked.Count);
        });

        [Fact]
        public Task Clean_up_finds_nothing_when_the_rom_folder_is_missing() => OnUi(() =>
        {
            string kept = Game("Kept (USA)", 1);
            Server.Games.Add(new FakeGame(1, "Kept", Md5(kept)));
            MainWindow window = Open();
            Pump(200);
            Scrape(window, ScrapeScope.ThisGame(kept));
            RunEnds(window);
            SettingsOf(window).RomDirectory = Path.Combine(Root, "Unplugged");
            Assert.Empty(window.OrphanedMedia());
            Refresh(window);
            Assert.Empty(window.OrphanedMedia());
            Assert.StartsWith("No orphaned media", window.CleanUpOrphansAsync().GetAwaiter().GetResult());
            Assert.Empty(_cleanUpAsked);
            Assert.True(File.Exists(Stored("snes", "covers", "Kept (USA).png")));
        });
    }
}
