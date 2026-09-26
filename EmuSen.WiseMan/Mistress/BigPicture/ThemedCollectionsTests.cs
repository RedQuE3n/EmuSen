using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Collections in the carousel, a gamelist's options and the random entry, driven by the pad over a themed session - see EmuSen_BigPicture.md §22.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedCollectionsTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedCollectionsTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public ThemedCollectionsTests(ITestOutputHelper output) => _out = output;

        internal static readonly Action<AppSettings> AllAuto = a => a.BigPictureCollections.AutoCollections =
            [BigPictureCollections.AllGames, BigPictureCollections.Favorites, BigPictureCollections.LastPlayed];

        private static Task Run(Action<ThemedSession> test, Action<AppSettings>? settings = null) => Session.Dispatch(() =>
        {
            using var session = new ThemedSession(settings: settings);
            test(session);
        }, default);

        internal static GameRecords Records(ThemedSession s) => (GameRecords)typeof(MainWindow).GetField("_records", Hidden)!.GetValue(s.Window)!;

        // What a change of records does in the window: the library, and with it the themed view, shown again.
        internal static void Refresh(ThemedSession s)
        {
            typeof(MainWindow).GetMethod("ShowLibraryEntries", Hidden)!.Invoke(s.Window, null);
            s.Settle();
        }

        internal static string Rom(ThemedSession s, string game) => Path.Combine(s.RomDirectory, game + (ThemedSession.SnesGames.Contains(game) ? ".sfc" : ThemedSession.NesGames.Contains(game) ? ".nes" : ".gb"));

        internal static string[] Systems(ThemedSession s) => s.Themed.Stage!.Current.Data.Systems.Select(x => x.System.Name).ToArray();

        internal static string[] Listed(ThemedSession s) => s.Themed.Stage!.Current.Data.System.Games.Select(g => g.Name).ToArray();

        // From the system view, right until the named system, then A.
        internal static void Enter(ThemedSession s, string system)
        {
            for (int guard = 0; guard < 12 && s.System != system; guard++) s.Pad.Right();
            Assert.Equal(system, s.System);
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
        }

        private static string[] MenuLines(ThemedSession s) =>
            s.Window.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        // Start, then down to the entry and A, as a player would.
        internal static void Choose(ThemedSession s, string entry)
        {
            s.Pad.Start();
            int at = Array.FindIndex(MenuLines(s), l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", MenuLines(s))}");
            s.Pad.Down(at);
            s.Pad.A();
            s.Settle();
        }

        internal static SheetLayer Sheets(ThemedSession s) => s.Window.GetControl<SheetLayer>("Sheets");

        internal static Control Sheet(ThemedSession s) => Sheets(s).SheetOf(Sheets(s).Current!)!;

        internal static T Named<T>(ThemedSession s, string name) where T : Control =>
            Sheet(s).GetVisualDescendants().OfType<T>().Single(c => c.Name == name);

        // The pad walks to the named control on the sheet showing now.
        internal static InputElement Reach(ThemedSession s, Func<InputElement, bool> target)
        {
            s.Window.UpdateLayout();
            return PadAudit.Reach(Sheet(s), s.Pad, target);
        }

        private static void Scraped(ThemedSession s, Dictionary<string, (string? Genre, float? Rating)> text)
        {
            var records = text.ToDictionary(p => Rom(s, p.Key), p => new ScrapedRecord("0", 0, ScrapeState.Found) { Genre = p.Value.Genre, Rating = p.Value.Rating });
            typeof(MainWindow).GetField("_scrapedText", Hidden)!.SetValue(s.Window, (IReadOnlyDictionary<string, ScrapedRecord>)records);
            Refresh(s);
        }

        [Fact]
        public Task Automatic_collections_are_off_until_turned_on_then_follow_the_systems_in_ES_DE_s_measured_order() => Session.Dispatch(() =>
        {
            using (var plain = new ThemedSession())
            {
                Records(plain).CreateCollection("Platform", DateTime.Now);
                Refresh(plain);
                Assert.Equal(["nes", "gb", "snes"], Systems(plain));
            }

            using var s = new ThemedSession(settings: AllAuto);
            GameRecords records = Records(s);
            long id = records.CreateCollection("Platform", DateTime.Now)!.Value;
            records.AddToCollection(id, Rom(s, ThemedSession.SnesGames[2]));
            records.ToggleFavourite(Rom(s, ThemedSession.SnesGames[1]));
            records.Started(Rom(s, ThemedSession.GbGames[0]), new DateTime(2026, 9, 2));
            Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "collections", "all", "favorites", "recent"], Systems(s));
            SceneData d = s.Themed.Stage!.Current.Data;
            Assert.Equal(["auto-allgames", "auto-favorites", "auto-lastplayed", "custom-collections"],
                new[] { "all", "favorites", "recent", "collections" }.Select(n => d.Systems.Single(x => x.System.Name == n).System.Theme));
            Assert.Equal(ThemeSystemKind.AutoCollection, d.Systems.Single(x => x.System.Name == "all").System.Kind);
            Assert.Equal(ThemeSystemKind.CustomCollection, d.Systems.Single(x => x.System.Name == "collections").System.Kind);
        }, default);

        private static readonly string MediaRoot = Path.Combine(Path.GetTempPath(), "EmuSenCollectionsTestMedia");

        [Fact]
        public Task All_games_favorites_and_last_played_come_from_Mistress_s_records_and_their_games_keep_their_own_system() => Session.Dispatch(() =>
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            using var s = new ThemedSession(settings: a => { AllAuto(a); a.EsdeMediaDirectory = MediaRoot; });
            GameRecords records = Records(s);
            records.ToggleFavourite(Rom(s, ThemedSession.SnesGames[3]));
            records.ToggleFavourite(Rom(s, ThemedSession.NesGames[1]));
            records.Started(Rom(s, ThemedSession.SnesGames[0]), new DateTime(2026, 9, 1));
            records.Started(Rom(s, ThemedSession.GbGames[0]), new DateTime(2026, 9, 3));
            records.Started(Rom(s, ThemedSession.NesGames[0]), new DateTime(2026, 9, 2));
            Refresh(s);

            Enter(s, "all");
            string[] all = Listed(s);
            Assert.Equal(8, all.Length);
            Assert.Equal(new[] { ThemedSession.SnesGames[3], ThemedSession.NesGames[1] }.Order(StringComparer.OrdinalIgnoreCase), all.Take(2));
            Assert.Equal(all.Skip(2), all.Skip(2).Order(StringComparer.OrdinalIgnoreCase));
            Assert.True(s.Themed.Stage!.Current.Data.System.Stars);
            // A game of a collection is looked up, and suffixed, under the system it belongs to.
            Assert.Equal("gb", s.Themed.Stage.Current.Data.System.Games.Single(g => g.Name == ThemedSession.GbGames[0]).SourceIn(s.Themed.Stage.Current.Data.System).Name);
            s.Pad.Down(Array.IndexOf(all, ThemedSession.GbGames[0]));
            Assert.Equal(ThemedSession.GbGames[0], s.Game);
            SceneBuilder scene = s.Themed.Stage.Current.Scene;
            string? cover = (scene.Find("image", "cover")!.Control as FittedImage)?.Source;
            Assert.Equal(Path.Combine(MediaRoot, "gb", "covers", ThemedSession.GbGames[0] + ".png"), cover);
            TextRowList rows = scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();
            Assert.Equal(ThemedSession.GbGames[0] + " [GB]", rows.Items![Array.IndexOf(all, ThemedSession.GbGames[0])].Text);
            s.Pad.L2();

            s.Pad.Right();
            Assert.Equal("favorites", s.System);
            Assert.Equal(new[] { ThemedSession.SnesGames[3], ThemedSession.NesGames[1] }.Order(StringComparer.OrdinalIgnoreCase), Listed(s));
            Assert.False(s.Themed.Stage.Current.Data.System.Stars);
            TextRowList list = s.Themed.Stage.Current.Scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();
            Assert.All(list.Items!, r => Assert.Equal(TextRowMarker.None, r.Marker));

            s.Pad.Right();
            Assert.Equal("recent", s.System);
            Assert.Equal([ThemedSession.GbGames[0], ThemedSession.NesGames[0], ThemedSession.SnesGames[0]], Listed(s));
        }, default);

        [Fact]
        public Task The_collection_settings_sheet_applies_each_switch_and_choice_beneath_it_at_once_and_saves_them() => Run(s =>
        {
            GameRecords records = Records(s);
            records.ToggleFavourite(Rom(s, ThemedSession.SnesGames[0]));
            long id = records.CreateCollection("Platform", DateTime.Now)!.Value;
            records.AddToCollection(id, Rom(s, ThemedSession.SnesGames[0]));
            Refresh(s);
            Choose(s, "Game Collection Settings");
            var settings = (AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(s.Window)!;

            Reach(s, e => e is LunaSwitch { Name: "Autoall" });
            s.Pad.A();
            Assert.Equal(["nes", "gb", "snes", "collections", "all"], Systems(s));
            Reach(s, e => e is LunaSwitch sw && sw.Name == $"Custom{id}");
            s.Pad.A();
            Assert.Equal(["nes", "gb", "snes", "all"], Systems(s));
            Assert.Equal([id], settings.BigPictureCollections.HiddenCustomCollections);
            s.Pad.A();
            Reach(s, e => e is Dropdown { Name: "CollectionGrouping" });
            s.Pad.Right(2);
            Assert.Equal(["nes", "gb", "snes", "Platform", "all"], Systems(s));
            Reach(s, e => e is LunaSwitch { Name: "FavoritesFirst" });
            s.Pad.A();
            Reach(s, e => e is Dropdown { Name: "DefaultSortOrder" });
            s.Pad.Right();
            Reach(s, e => e is Dropdown { Name: "RandomEntryButton" });
            s.Pad.Right(2);
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);

            BigPictureCollections saved = AppSettings.Load().BigPictureCollections;
            Assert.Equal((BigPictureCollections.GroupNever, BigPictureCollections.RandomDisabled, "name, descending", false),
                (saved.GroupCustomCollections, saved.RandomEntryButton, saved.DefaultSortOrder, saved.FavoritesFirst));
            Assert.Equal([BigPictureCollections.AllGames], saved.AutoCollections);
            Assert.Empty(saved.HiddenCustomCollections);
            Enter(s, "snes");
            Assert.Equal(ThemedSession.SnesGames.Reverse(), Listed(s));
            string before = s.Game!;
            s.Pad.L3();
            Assert.Equal(before, s.Game);
        });

        [Fact]
        public Task The_grouped_collections_system_lists_each_collection_as_a_folder_that_A_enters_and_B_leaves() => Run(s =>
        {
            GameRecords records = Records(s);
            long platform = records.CreateCollection("Platform", DateTime.Now)!.Value, beat = records.CreateCollection("Beat", DateTime.Now)!.Value;
            foreach (string g in new[] { ThemedSession.SnesGames[4], ThemedSession.SnesGames[0], ThemedSession.SnesGames[2] }) records.AddToCollection(platform, Rom(s, g));
            records.AddToCollection(beat, Rom(s, ThemedSession.NesGames[1]));
            records.AddToCollection(beat, Rom(s, ThemedSession.SnesGames[1]));
            Refresh(s);
            s.Themed.Random = new Random(5);

            Enter(s, "collections");
            Assert.Equal(["Beat", "Platform"], Listed(s));
            SceneGame folder = s.Themed.SelectedGame!;
            Assert.True(folder.Folder);
            Assert.Matches(@"^This collection contains 2 games: '.+ \[(NES|SNES)\]' and '.+ \[(NES|SNES)\]'$", folder.Description);
            Assert.Contains($"'{ThemedSession.NesGames[1]} [NES]'", folder.Description);
            Assert.StartsWith($"This collection contains 2 games: '{folder.Face!.Name} [", folder.Description);

            s.Pad.Down();
            s.Sounds.Clear();
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
            Assert.True(s.Themed.InFolder);
            Assert.Equal([ThemedSession.SnesGames[0], ThemedSession.SnesGames[2], ThemedSession.SnesGames[4]], Listed(s));
            Assert.Equal(["select"], s.Sounds);
            Assert.Equal(ThemeSystemKind.CustomCollection, s.Themed.SelectedSystem!.System.Kind);
            Assert.Equal("snes", s.Themed.SelectedGame!.SourceIn(s.Themed.Stage!.Current.Data.System).Name);

            s.Pad.B();
            Assert.Equal("gamelist", s.View);
            Assert.False(s.Themed.InFolder);
            Assert.Equal("Platform", s.Game);
            Assert.Equal(["select", "back"], s.Sounds);
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Equal("collections", s.System);
        });

        private const string MetadataElements =
            "<rating name=\"r\"><pos>0.55 0.8</pos><size>0 0.05</size></rating>" +
            "<text name=\"sys\"><pos>0.55 0.85</pos><size>0.4 0.05</size><metadata>systemFullname</metadata><color>FFFFFF</color></text>" +
            "<text name=\"desc\"><pos>0.55 0.9</pos><size>0.4 0.05</size><metadata>description</metadata><color>FFFFFF</color></text>" +
            "<text name=\"dev\"><pos>0.55 0.95</pos><size>0.4 0.05</size><metadata>developer</metadata><defaultValue>unknown</defaultValue><color>FFFFFF</color></text>";

        [Fact]
        public Task A_collection_s_entry_is_drawn_as_ES_DE_draws_it_no_folder_mark_its_metadata_hidden_and_a_blank_system_name() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(extraGamelist: MetadataElements);
            long platform = Records(s).CreateCollection("Platform", DateTime.Now)!.Value;
            Records(s).AddToCollection(platform, Rom(s, ThemedSession.SnesGames[0]));
            Refresh(s);
            Enter(s, "collections");
            SceneBuilder scene = s.Themed.Stage!.Current.Scene;
            Assert.True(s.Themed.SelectedGame!.IsCollection);
            Assert.Equal(TextRowMarker.None, scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single().Items![0].Marker);
            Assert.Equal("the entry hides its metadata", scene.Find("rating", "r")!.Skipped);
            Assert.Equal("the entry hides its metadata", scene.Find("text", "dev")!.Skipped);
            Assert.NotNull(scene.Find("text", "desc")!.Control);
            Assert.Null(scene.Find("text", "sys")!.Control);
            Assert.Contains(scene.Entries.Select(e => e.Control).OfType<HintBar>().Single().Entries!, h => h.Label == "Select");

            // Inside, a game again: its metadata drawn, and the system name is the collection's.
            s.Pad.A();
            scene = s.Themed.Stage!.Current.Scene;
            Assert.NotEqual("the entry hides its metadata", scene.Find("rating", "r")!.Skipped);
            Assert.Equal("unknown", ((FontText)scene.Find("text", "dev")!.Control!).Text);
            Assert.Equal("Platform", ((FontText)scene.Find("text", "sys")!.Control!).Text);
            Assert.Contains(scene.Entries.Select(e => e.Control).OfType<HintBar>().Single().Entries!, h => h.Label == "Launch");
        }, default);

        [Fact]
        public Task A_collection_the_theme_has_a_folder_for_is_a_system_of_its_own_and_the_setting_can_group_it_or_part_all() => Run(s =>
        {
            GameRecords records = Records(s);
            long platform = records.CreateCollection("Platform", DateTime.Now)!.Value, beat = records.CreateCollection("Beat", DateTime.Now)!.Value;
            records.AddToCollection(platform, Rom(s, ThemedSession.SnesGames[0]));
            records.AddToCollection(beat, Rom(s, ThemedSession.SnesGames[1]));
            s.Theme.File("Platform/theme.xml", "<theme><include>./../theme.xml</include></theme>");
            Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "collections", "Platform"], Systems(s));

            var settings = (AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(s.Window)!;
            settings.BigPictureCollections.GroupCustomCollections = BigPictureCollections.GroupAlways;
            Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "collections"], Systems(s));
            settings.BigPictureCollections.GroupCustomCollections = BigPictureCollections.GroupNever;
            Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "Beat", "Platform"], Systems(s));
            settings.BigPictureCollections.HiddenCustomCollections.Add(beat);
            Refresh(s);
            Assert.Equal(["nes", "gb", "snes", "Platform"], Systems(s));
        });

        [Fact]
        public Task A_collection_created_in_big_picture_is_the_library_s_own_and_North_edits_it_until_its_editing_is_finished() => Run(s =>
        {
            Choose(s, "Game Collection Settings");
            Assert.IsType<CollectionSettingsWindow>(Sheets(s).Current);
            Reach(s, e => e is Button { Name: "CollectionCreate" });
            s.Pad.A();
            s.Settle();
            Assert.Equal("PromptWindow", Sheets(s).Current!.GetType().Name);
            Named<TextBox>(s, "PART_Answer").Text = "Beat: Up*";
            Reach(s, e => e is Button { Name: "PART_Accept" });
            s.Pad.A();
            s.Settle();

            // The name loses ES-DE's forbidden characters, the settings sheet closes into the edit mode, and the sidebar has the same collection.
            Assert.False(Sheets(s).IsPresenting);
            GameCollection made = Records(s).Collections().Single();
            Assert.Equal("Beat Up", made.Name);
            Assert.Equal("Beat Up", s.Themed.Editing!.Name);
            var sidebar = (IReadOnlyList<GameCollection>)typeof(MainWindow).GetField("_collections", Hidden)!.GetValue(s.Window)!;
            Assert.Equal(made.Id, sidebar.Single().Id);

            Enter(s, "snes");
            s.Pad.Down(2);
            string chosen = ThemedSession.SnesGames[2];
            s.Pad.Y();
            Assert.Contains(made.Id, Records(s).CollectionsOf(Rom(s, chosen)));
            Assert.Null(OnScreenKeyboard.OpenOver(s.Window));
            Assert.Equal(chosen, s.Game);
            Assert.True(s.Themed.SelectedGame!.InCollection);
            TextRowList list = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();
            Assert.Equal(TextRowMarker.Tick, list.Items![2].Marker);
            Assert.Equal(TextRowMarker.None, list.Items![1].Marker);
            Assert.True(s.Themed.Stage.Current.Data.Help.Editing);

            // The collection now shows in the carousel, grouped, holding the game.
            s.Pad.B();
            for (int guard = 0; guard < 8 && s.System != "collections"; guard++) s.Pad.Right();
            Assert.Equal("collections", s.System);
            s.Pad.A();
            s.Pad.A();
            Assert.Equal([chosen], Listed(s));

            // North again takes it out; then Finish Editing in the options, and North searches as before.
            s.Pad.Y();
            Assert.Empty(Records(s).Members(made.Id));
            Choose(s, "Gamelist Options");
            Reach(s, e => e is Button { Name: "GamelistFinishEditing" });
            s.Pad.A();
            s.Settle();
            Assert.Null(s.Themed.Editing);
            Assert.False(s.Themed.Stage!.Current.Data.Help.Editing);
            s.Pad.B();
            s.Pad.B();
            Enter(s, "snes");
            s.Pad.Y();
            Assert.NotNull(OnScreenKeyboard.OpenOver(s.Window));

            // A second collection of the same name is numbered, as ES-DE numbers it.
            Assert.Equal("Beat Up (1)", CollectionShelves.Unique("Beat Up", Records(s).Collections().Select(c => c.Name)));
        });

        [Fact]
        public Task The_options_sheet_sorts_filters_and_jumps_B_applying_and_Back_cancelling_and_each_list_keeps_its_own() => Run(s =>
        {
            Scraped(s, new()
            {
                [ThemedSession.SnesGames[0]] = ("Racing", 0.8f), [ThemedSession.SnesGames[1]] = ("Puzzle", 0.4f),
                [ThemedSession.SnesGames[2]] = ("Racing", null), [ThemedSession.SnesGames[4]] = ("Shooter", 1f),
            });
            Enter(s, "snes");
            string[] byName = Listed(s);

            // Back cancels: the sort is stepped, then the Back button closes the sheet with nothing applied.
            Choose(s, "Gamelist Options");
            Assert.IsType<GamelistOptionsWindow>(Sheets(s).Current);
            Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Right();
            s.Pad.Select();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Equal(byName, Listed(s));

            // B applies: name, descending.
            Choose(s, "Gamelist Options");
            Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Right();
            s.Pad.B();
            Assert.False(Sheets(s).IsPresenting);
            Assert.Equal(byName.Reverse(), Listed(s));
            Assert.Equal(new GameSort(GameSortKey.Name, true), s.Themed.CurrentSort);

            // Rating, descending, from ScreenScraper's text: a game without a rating goes last.
            Choose(s, "Gamelist Options");
            Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Right(2);
            s.Pad.B();
            Assert.Equal(new GameSort(GameSortKey.Rating, true), s.Themed.CurrentSort);
            Assert.Equal([ThemedSession.SnesGames[4], ThemedSession.SnesGames[0], ThemedSession.SnesGames[1], ThemedSession.SnesGames[2], ThemedSession.SnesGames[3]], Listed(s));

            // The filter screen: Racing alone.
            Choose(s, "Gamelist Options");
            Reach(s, e => e is Button { Name: "GamelistFilterButton" });
            s.Pad.A();
            s.Settle();
            Assert.IsType<GamelistFilterWindow>(Sheets(s).Current);
            Assert.Equal(["Puzzle", "Racing", "Shooter", GamelistOptions.Unknown], Named<WrapPanel>(s, "FilterGenreValues").Children.OfType<LunaSwitch>().Select(b => b.Label));
            Assert.Equal(GamelistOptions.NothingToFilter, Named<HintText>(s, "FilterPublisherNothing").Text);
            Reach(s, e => e is LunaSwitch { Label: "Racing" });
            s.Pad.A();
            s.Pad.B();
            s.Settle();
            Assert.IsType<GamelistOptionsWindow>(Sheets(s).Current);
            Assert.Equal("1 filter set", Named<TextBlock>(s, "GamelistFilterState").Text);
            s.Pad.B();
            Assert.Equal([ThemedSession.SnesGames[0], ThemedSession.SnesGames[2]], Listed(s));

            // Another system keeps its own order and no filter.
            s.Pad.Left();
            Assert.Equal("gb", s.System);
            s.Pad.Left();
            Assert.Equal("nes", s.System);
            Assert.Equal(ThemedSession.NesGames, Listed(s));
            s.Pad.Right(2);
            Assert.Equal([ThemedSession.SnesGames[0], ThemedSession.SnesGames[2]], Listed(s));

            // The filter is reset on its screen; the jump goes to the first game of the letter chosen.
            Choose(s, "Gamelist Options");
            Reach(s, e => e is Button { Name: "GamelistFilterButton" });
            s.Pad.A();
            s.Settle();
            Reach(s, e => e is Button { Name: "FilterReset" });
            s.Pad.A();
            s.Pad.B();
            s.Settle();
            Reach(s, e => e is Dropdown { Name: "GamelistSortBy" });
            s.Pad.Left(3);
            s.Pad.B();
            Assert.Equal(byName, Listed(s));
            Assert.Equal(GameSort.Default, s.Themed.CurrentSort);

            Choose(s, "Gamelist Options");
            Reach(s, e => e is Dropdown { Name: "GamelistJumpTo" });
            Assert.Equal(new[] { "A", "B", "C", "D", "E" }, Named<Dropdown>(s, "GamelistJumpTo").Items.Cast<object>().Select(o => o.ToString()));
            Assert.Equal(GamelistOptions.FirstLetter(s.Game!), Named<Dropdown>(s, "GamelistJumpTo").SelectedItem as string);
            for (int guard = 0; guard < 5 && Named<Dropdown>(s, "GamelistJumpTo").SelectedItem as string != "D"; guard++) s.Pad.Right();
            s.Pad.B();
            Assert.Equal(ThemedSession.SnesGames[3], s.Game);

            // Nothing of it is saved: the settings file holds no sort or filter.
            Assert.DoesNotContain("Racing", File.ReadAllText(Path.Combine(s.Root, "Config", "appsettings.json")));
        });

        [Fact]
        public Task Either_stick_pressed_in_jumps_to_another_game_and_the_setting_decides_where_it_works() => Run(s =>
        {
            s.Themed.Random = new Random(11);
            s.Pad.L3();
            Assert.Equal("nes", s.System);
            Enter(s, "snes");
            Assert.Contains(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Single().Entries!, h => h.Label == "Random");
            var seen = new HashSet<string>();
            for (int i = 0; i < 12; i++)
            {
                string before = s.Game!;
                s.Sounds.Clear();
                if (i % 2 == 0) s.Pad.L3();
                else s.Pad.R3();
                Assert.NotEqual(before, s.Game);
                Assert.Equal(["scroll"], s.Sounds);
                seen.Add(s.Game!);
            }
            Assert.True(seen.Count >= 4, string.Join(", ", seen));

            var settings = (AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(s.Window)!;
            settings.BigPictureCollections.RandomEntryButton = BigPictureCollections.RandomDisabled;
            Refresh(s);
            string still = s.Game!;
            s.Pad.L3();
            Assert.Equal(still, s.Game);
            Assert.DoesNotContain(s.Themed.Stage.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Single().Entries!, h => h.Label == "Random");

            settings.BigPictureCollections.RandomEntryButton = BigPictureCollections.RandomGamesAndSystems;
            Refresh(s);
            s.Pad.B();
            string system = s.System!;
            s.Pad.R3();
            Assert.NotEqual(system, s.System);
            Assert.Equal("system", s.View);
        });

        [Theory]
        [InlineData("Gamelist Options")]
        [InlineData("Filter Gamelist")]
        [InlineData("Game Collection Settings")]
        public Task Every_control_of_the_collection_sheets_is_reached_by_the_pad(string sheet) => Run(s =>
        {
            GameRecords records = Records(s);
            long id = records.CreateCollection("Platform", DateTime.Now)!.Value;
            records.AddToCollection(id, Rom(s, ThemedSession.SnesGames[0]));
            Scraped(s, new() { [ThemedSession.SnesGames[0]] = ("Racing", 0.8f), [ThemedSession.SnesGames[1]] = ("Puzzle", null) });
            if (sheet == "Game Collection Settings") Choose(s, sheet);
            else
            {
                Enter(s, sheet == "Filter Gamelist" ? "snes" : "collections");
                if (sheet != "Filter Gamelist") s.Pad.A();
                Choose(s, "Gamelist Options");
                if (sheet == "Filter Gamelist")
                {
                    Reach(s, e => e is Button { Name: "GamelistFilterButton" });
                    s.Pad.A();
                    s.Settle();
                }
            }
            Assert.Empty(s.Window.OwnedWindows);
            Control root = Sheet(s);
            s.Window.UpdateLayout();
            HashSet<InputElement> reached = PadAudit.Reachable(root, s.Pad);
            List<string> missing = PadAudit.Operable(root).Where(c => !reached.Contains(c)).Select(PadAudit.Describe).ToList();
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);
            _out.WriteLine($"{sheet}: {reached.Count} controls reached");
        }, AllAuto);
    }
}
