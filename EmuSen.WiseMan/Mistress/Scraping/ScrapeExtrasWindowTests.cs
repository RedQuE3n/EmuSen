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

        private static void ShowFindByName(MainWindow w, string path) =>
            typeof(MainWindow).GetMethod("ShowFindByName", Hidden)!.Invoke(w, [path, Path.GetFileNameWithoutExtension(path)]);

        private static void Search(FindByNameWindow find, string text)
        {
            Named<TextBox>(find, "FindByNameText").Text = text;
            Task searching = find.SearchAsync();
            WaitFor(() => searching.IsCompleted, "the search");
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
        public Task Find_by_name_sends_nothing_until_Search_then_one_request_and_a_pick_keeps_that_game_with_no_lookup() => OnUi(() =>
        {
            string hack = Game("SMW Hack (USA) [Hack]", 5);
            Server.Games.Add(new FakeGame(900, "Super Mario World"));
            Server.Games.Add(new FakeGame(901, "Super Mario Kart"));
            MainWindow window = Open();
            Pump(200);

            ShowFindByName(window, hack);
            FindByNameWindow find = Assert.IsType<FindByNameWindow>(window.FindByNameShown);
            Pump(200);
            Assert.Empty(Server.Asked);
            Assert.Equal("SMW Hack", Named<TextBox>(find, "FindByNameText").Text);

            Search(find, "Super Mario");
            string asked = Assert.Single(Server.Asked);
            Assert.Contains("/jeuRecherche.php", asked);
            Assert.Equal(("4", "Super Mario"), (FakeScreenScraper.Param(asked, "systemeid"), FakeScreenScraper.Param(asked, "recherche")));
            Assert.Equal(1, window.NameSearches);
            Assert.Equal(["FindByNameResult_0", "FindByNameResult_1"], find.Results.Select(b => b.Name));
            Assert.StartsWith("2 games", find.StatusText);

            Click(find.Results.First(b => ((ScrapedGame)b.Tag!).Id == 900));
            Assert.Null(window.FindByNameShown);
            RunEnds(window);
            Assert.Empty(Server.JeuInfos);
            Assert.Equal(4, Server.MediaAsked.Count());
            Assert.Equal(5, Server.Asked.Count);
            ScrapedRecord kept = window.ScrapeStore!.FoundFor(hack)!;
            Assert.Equal((900L, "name", "Super Mario World (US title)"), (kept.GameId!.Value, kept.MatchedBy, kept.Name));
            Pump(200);
            Assert.Equal(Stored("snes", "covers", "SMW Hack (USA) [Hack].png"), CoverShown(window, hack));
            // The file is never renamed, and nothing is written beside it.
            Assert.Equal([hack], Directory.EnumerateFiles(RomDir));
        });

        [Fact]
        public Task A_search_that_finds_nothing_says_so_and_counts_against_the_unrecognised_allowance() => OnUi(() =>
        {
            string rom = Game("Nameless", 9);
            MainWindow window = Open();
            ShowFindByName(window, rom);
            FindByNameWindow find = window.FindByNameShown!;

            Search(find, "Nothing Like It");

            Assert.Single(Server.Searches);
            Assert.Empty(find.Results);
            Assert.Contains("found no game", find.StatusText);
            QuotaSnapshot quota = ((IScrapeHost)window).Quota!;
            Assert.Equal((11, 2), (quota.RequestsToday, quota.KoToday));
        });

        [Fact]
        public Task Without_the_developer_file_find_by_name_says_why_and_sends_nothing() => OnUi(() =>
        {
            string rom = Game("Nameless", 9);
            MainWindow window = Open(developer: false);
            ShowFindByName(window, rom);
            FindByNameWindow find = window.FindByNameShown!;

            Assert.False(Named<Button>(find, "FindByNameSearch").IsEnabled);
            Assert.StartsWith("ScreenScraper cannot be asked", find.StatusText);
            Task searching = find.SearchAsync();
            WaitFor(() => searching.IsCompleted, "the search");
            Assert.Empty(Server.Asked);
            Assert.Equal(0, window.NameSearches);
        });

        [Fact]
        public Task Find_by_name_is_in_the_game_s_options_and_the_context_menu_and_neither_offers_it_during_a_run() => OnUi(() =>
        {
            string rom = Game("F-Zero (USA)", 1);
            Server.Games.Add(new FakeGame(77, "F-Zero", Md5(rom)));
            MainWindow window = Open();
            Pump(200);
            RomEntry entry = DesktopGameOptionsTests.List(window).Models.Single(e => e.FullPath == rom);

            typeof(MainWindow).GetMethod("ShowLibraryGameOptions", Hidden)!.Invoke(window, [entry]);
            GameOptionsWindow options = Assert.IsType<GameOptionsWindow>(window.GameOptionsShown);
            Button entryButton = options.Entries.Single(b => b.Content as string == "Find by Name...");
            Click(entryButton);
            Pump(100);
            Assert.NotNull(window.FindByNameShown);
            window.FindByNameShown!.Close();
            Assert.Empty(Server.Asked);

            LunaList<RomEntry> list = DesktopGameOptionsTests.List(window);
            Control row = list.ContainerFromIndex(list.Models.ToList().IndexOf(entry))!;
            Point centre = row.TranslatePoint(new Point(row.Bounds.Width / 2, row.Bounds.Height / 2), window)!.Value;
            window.MouseDown(centre, MouseButton.Right);
            window.MouseUp(centre, MouseButton.Right);
            Pump(100);
            ContextMenu menu = list.ContextMenu!;
            string[] labels = menu.ItemsSource!.OfType<ActionMenuItem>().Select(i => i.Action.Text).ToArray();
            ActionMenuItem find = menu.ItemsSource!.OfType<ActionMenuItem>().SingleOrDefault(i => i.Action.Text == "_Find by Name...") ?? throw new InvalidOperationException(string.Join(" | ", labels));
            Assert.True(find.IsEnabled);
            menu.Close();
            find.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
            Pump(100);
            Assert.NotNull(window.FindByNameShown);
            window.FindByNameShown!.Close();

            Server.Gate = new System.Threading.SemaphoreSlim(0);
            Scrape(window, ScrapeScope.ThisGame(rom));
            typeof(MainWindow).GetMethod("ShowLibraryGameOptions", Hidden)!.Invoke(window, [entry]);
            Assert.DoesNotContain(window.GameOptionsShown!.Entries, b => b.Content as string == "Find by Name...");
            window.GameOptionsShown!.Close();
            window.CancelScrape();
            Assert.Empty(Server.Searches);
        });

        [Fact]
        public Task On_a_sheet_the_pad_reaches_the_box_Search_and_every_result() => OnUi(() =>
        {
            string rom = Game("SMW Hack (USA)", 5);
            Server.Games.Add(new FakeGame(900, "Super Mario World"));
            Server.Games.Add(new FakeGame(901, "Super Mario Kart"));
            MainWindow window = Open(bigScreen: true);
            var pad = new PadDriver(window);
            Pump(200);
            ShowFindByName(window, rom);
            FindByNameWindow find = Assert.IsType<FindByNameWindow>(Sheets(window).Current);
            Control sheet = RootOf(find);
            window.UpdateLayout();

            HashSet<InputElement> reached = PadAudit.Reachable(sheet, pad);
            Assert.Contains(reached, e => e is TextBox { Name: "FindByNameText" });
            Assert.Contains(reached, e => e is Button { Name: "FindByNameSearch" });
            Assert.Empty(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));

            Named<TextBox>(find, "FindByNameText").Text = "Super Mario";
            PadAudit.Reach(sheet, pad, e => e is Button { Name: "FindByNameSearch" });
            pad.A();
            WaitFor(() => find.Results.Count() == 2, "the results");
            window.UpdateLayout();
            reached = PadAudit.Reachable(sheet, pad);
            Assert.Contains(reached, e => e is Button { Name: "FindByNameResult_1" });
            Assert.Empty(PadAudit.Operable(sheet).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));
            PadAudit.Reach(sheet, pad, e => e is Button { Name: "FindByNameResult_1" });
            pad.A();
            RunEnds(window);
            Assert.Equal(901L, window.ScrapeStore!.FoundFor(rom)!.GameId);
        });

        // --- the criteria and the plan ---

        [Fact]
        public Task Each_criterion_chooses_the_games_a_wide_run_takes() => OnUi(() =>
        {
            string scraped = Game("Scraped (USA)", 1), favourite = Game("Favourite (USA)", 3), shot = Game("Shot Only (USA)", 5), clip = Game("Clip Only (USA)", 7), bare = Game("Bare (USA)", 9);
            Server.Games.Add(new FakeGame(1, "Scraped", Md5(scraped)));
            MainWindow window = Open();
            Pump(200);
            Scrape(window, ScrapeScope.ThisGame(scraped));
            RunEnds(window);
            Pump(200);
            RecordsOf(window).ToggleFavourite(favourite);
            Directory.CreateDirectory(Stored("snes", "screenshots"));
            File.WriteAllBytes(Stored("snes", "screenshots", "Shot Only (USA).png"), new byte[100]);
            Directory.CreateDirectory(Stored("snes", "videos"));
            File.WriteAllBytes(Stored("snes", "videos", "Clip Only (USA).mp4"), new byte[100]);
            Refresh(window);

            int Count(ScrapeCriteria c) => window.Plan(new ScrapeScope { Criteria = c }).Games;
            Assert.Equal(5, Count(ScrapeCriteria.All));
            Assert.Equal(1, Count(ScrapeCriteria.Favourites));
            Assert.Equal(4, Count(ScrapeCriteria.NoMetadata));
            Assert.Equal(4, Count(ScrapeCriteria.NoCover));
            Assert.Equal(3, Count(ScrapeCriteria.NoGameImage));
            Assert.Equal(4, Count(ScrapeCriteria.NoGameVideo));
            Assert.Equal(4, window.Plan(new ScrapeScope(MissingArtOnly: true)).Games);
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
        public Task The_scraping_tab_offers_each_new_kind_off_the_criteria_and_refresh_and_saves_them() => OnUi(() =>
        {
            MainWindow window = Open();
            var prefs = new PreferencesWindow(SettingsOf(window), window);
            Windows.Add(prefs);
            prefs.Show();
            prefs.ShowTab(PreferencesWindow.ScrapingTab);
            Pump(100);

            string[] kinds = ["ScrapeBackCoversSwitch", "Scrape3DBoxesSwitch", "ScrapePhysicalMediaSwitch", "ScrapeFanArtSwitch", "ScrapeManualsSwitch", "ScrapeVideosSwitch", "ScrapeGameNamesSwitch", "ScrapeRefreshSwitch"];
            foreach (string name in kinds) Assert.False(Named<LunaSwitch>(prefs, name).IsChecked, name);
            Assert.Equal("Games with no cover", Named<Dropdown>(prefs, "ScrapeCriteriaDropdown").SelectedItem);

            foreach (string name in kinds) Named<LunaSwitch>(prefs, name).IsChecked = true;
            Named<Dropdown>(prefs, "ScrapeCriteriaDropdown").SelectedItem = "Favourite games";
            Pump(100);
            AppSettings saved = AppSettings.Load();
            Assert.True(saved.ScrapeBackCovers && saved.Scrape3DBoxes && saved.ScrapePhysicalMedia && saved.ScrapeFanArt && saved.ScrapeManuals && saved.ScrapeVideos && saved.ScrapeGameNames && saved.ScrapeRefresh);
            Assert.Equal("favorites", saved.ScrapeCriteria);
            Assert.Empty(Server.Asked);
        });

        // --- ScreenScraper's names (Q30) ---

        [Fact]
        public Task Game_names_shows_ScreenScraper_s_name_only_when_on_and_the_player_s_name_still_wins() => OnUi(() =>
        {
            string rom = Game("F-Zero (USA)", 1);
            Server.Games.Add(new FakeGame(77, "F-Zero", Md5(rom)));
            MainWindow window = Open();
            Pump(200);
            Scrape(window, ScrapeScope.ThisGame(rom));
            RunEnds(window);
            Pump(200);
            Assert.Contains("F-Zero (USA)", DesktopGameOptionsTests.ListLabels(window));

            SettingsOf(window).ScrapeGameNames = true;
            Refresh(window);
            Assert.Contains("F-Zero (US title)", DesktopGameOptionsTests.ListLabels(window));
            MetadataDraft draft = ((IGameEditorHost)window).DraftFor(rom);
            Assert.Equal(("F-Zero (US title)", MetadataSource.Scraped), (draft.Value(GameMetadata.Name), draft.Source(GameMetadata.Name)));
            Assert.Null(draft.OfferedName);

            RecordsOf(window).SaveEdits(rom, new Dictionary<string, string?> { [GameMetadata.Name] = "My Racer" }, DateTime.Now);
            Refresh(window);
            Assert.Contains("My Racer", DesktopGameOptionsTests.ListLabels(window));

            SettingsOf(window).ScrapeGameNames = false;
            RecordsOf(window).ClearEdits(rom);
            Refresh(window);
            Assert.Contains("F-Zero (USA)", DesktopGameOptionsTests.ListLabels(window));
            Assert.Equal("F-Zero (US title)", ((IGameEditorHost)window).DraftFor(rom).OfferedName);
        });

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
            Assert.NotEmpty(window.ScrapeStore!.Media(goneMd5, goneBytes));
            File.Delete(gone);
            Refresh(window);
            var roms = Fingerprint(RomDir);
            int requests = Server.Asked.Count;

            IReadOnlyList<string> orphans = window.OrphanedMedia();
            Assert.Equal(4, orphans.Count);
            Assert.All(orphans, o => Assert.Contains("Gone (USA)", o));

            _cleanUpAnswer = false;
            Assert.Equal("Nothing was moved.", window.CleanUpOrphansAsync().GetAwaiter().GetResult());
            Assert.StartsWith("4 scraped file(s)", Assert.Single(_cleanUpAsked));
            Assert.True(File.Exists(Stored("snes", "covers", "Gone (USA).png")));

            _cleanUpAnswer = true;
            string said = window.CleanUpOrphansAsync().GetAwaiter().GetResult();
            Assert.StartsWith("Moved 4 file(s)", said);
            string backup = Assert.Single(Directory.GetDirectories(Stored(MediaStore.CleanupFolder)));
            Assert.True(File.Exists(Path.Combine(backup, "snes", "covers", "Gone (USA).png")));
            Assert.True(File.Exists(Path.Combine(backup, "cleanup.txt")));
            Assert.False(File.Exists(Stored("snes", "covers", "Gone (USA).png")));
            Assert.True(File.Exists(Stored("snes", "covers", "Kept (USA).png")));
            Assert.True(File.Exists(Stored("nes", "covers", "Elsewhere.png")));
            Assert.Empty(window.ScrapeStore!.Media(goneMd5, goneBytes));
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
            Assert.StartsWith("No orphaned media", window.CleanUpOrphansAsync().GetAwaiter().GetResult());
            Assert.Empty(_cleanUpAsked);
            Assert.True(File.Exists(Stored("snes", "covers", "Kept (USA).png")));
        });
    }
}
