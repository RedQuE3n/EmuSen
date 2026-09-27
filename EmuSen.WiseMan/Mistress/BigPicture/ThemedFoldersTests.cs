using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using EmuSen.Mistress.Scraping;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Folders in the themed gamelist and the media of games inside them - see EmuSen_BigPicture.md §30.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedFoldersTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedFoldersTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly ITestOutputHelper _out;

        public ThemedFoldersTests(ITestOutputHelper output) => _out = output;

        internal const string UsaGame = "Tidal Keep (Synthetic)", EuropeGame = "Quartz Mill (Synthetic)";

        // The NES shape decided in §21.1: a console folder, then region folders, and no ROM at the console folder's top.
        internal static void RegionFolders(string roms)
        {
            Directory.CreateDirectory(Path.Combine(roms, "NES", "USA"));
            Directory.CreateDirectory(Path.Combine(roms, "NES", "Europe"));
            File.WriteAllBytes(Path.Combine(roms, "NES", "USA", UsaGame + ".nes"), new byte[64]);
            File.WriteAllBytes(Path.Combine(roms, "NES", "Europe", EuropeGame + ".nes"), new byte[80]);
        }

        internal const string AmberVale = "Amber Vale (Synthetic)", DeepHack = "Deep Hack (Synthetic)", NeonLoop = "Neon Loop (Synthetic)", VelvetRally = "Velvet Rally (Synthetic)",
            AmberPulse = "Amber Pulse (Synthetic)", BirchEcho = "Birch Echo (Synthetic)";

        // A library of folders beside the session's own top-level games: NES by region with a nested hack, GB by letter, one SNES folder of games that start.
        internal static void Library(string roms)
        {
            RegionFolders(roms);
            void Write(string relative, byte[] bytes)
            {
                string path = Path.Combine(roms, relative.Replace('/', Path.DirectorySeparatorChar));
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllBytes(path, bytes);
            }
            Write($"NES/USA/{AmberVale}.nes", new byte[72]);
            Write($"NES/Hacks/Mario/{DeepHack}.nes", new byte[96]);
            Write($"GB/A/{AmberPulse}.gb", new byte[0x200]);
            Write($"GB/B/{BirchEcho}.gb", new byte[0x210]);
            Write($"SNES/Racing/{NeonLoop}.sfc", SyntheticRom.BuildBlank());
            Write($"SNES/Racing/{VelvetRally}.sfc", SyntheticRom.BuildBlank());
        }

        private static Task Run(Action<ThemedSession> test, Action<AppSettings>? settings = null) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(settings: settings, roms: Library);
            test(s);
        }, default);

        private static string[] Listed(ThemedSession s) => ThemedCollectionsTests.Listed(s);

        private static IReadOnlyList<SceneGame> Entries(ThemedSession s) => s.Themed.Stage!.Current.Data.System.Games;

        private static void Now(ThemedSession s, Action<TimeSpan> act)
        {
            act(s.Now);
            s.Settle();
        }

        private static void Stop(MainWindow window) => typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);

        private static string? Running(MainWindow w) => (string?)typeof(MainWindow).GetField("_currentRomPath", Hidden)!.GetValue(w);

        private static string[] NesTop => ["Europe", "Hacks", "USA", ThemedSession.NesGames[0], ThemedSession.NesGames[1]];

        [Fact]
        public Task NES_opens_on_its_folders_on_top_South_enters_East_leaves_and_each_level_keeps_its_place() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "nes");
            Assert.Equal(NesTop, Listed(s));
            Assert.Equal([true, true, true, false, false], Entries(s).Select(g => g.Folder));
            Assert.Equal(["Europe", "Hacks", "USA", "", ""], Entries(s).Select(g => g.FolderPath));
            Assert.Equal(Path.Combine(s.RomDirectory, "NES", "USA"), Entries(s)[2].File);
            Assert.Equal(Path.Combine(s.RomDirectory, "NES", "Hacks"), Entries(s)[1].File);
            TextRowList rows = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<TextRowList>().Single();
            Assert.Equal([TextRowMarker.Folder, TextRowMarker.Folder, TextRowMarker.Folder, TextRowMarker.None, TextRowMarker.None], rows.Items!.Select(r => r.Marker));
            Assert.True(rows.Items![0].Secondary);
            Assert.Equal("Select", HelpLabel(s, "a"));

            s.Pad.Down(2);
            s.Sounds.Clear();
            s.Pad.A();
            Assert.Equal(["select"], s.Sounds);
            Assert.Equal("gamelist", s.View);
            Assert.True(s.Themed.InFolder);
            Assert.Equal("USA", s.Themed.CurrentFolder);
            Assert.Equal([AmberVale, UsaGame], Listed(s));
            Assert.Equal("Launch", HelpLabel(s, "a"));
            s.Pad.Down();
            Assert.Equal(UsaGame, s.Game);

            s.Sounds.Clear();
            s.Pad.B();
            Assert.Equal(["back"], s.Sounds);
            Assert.Equal("gamelist", s.View);
            Assert.Equal("", s.Themed.CurrentFolder);
            Assert.Equal("USA", s.Game);
            s.Pad.A();
            Assert.Equal(UsaGame, s.Game);
            s.Pad.B();

            // A folder of folders, two levels down, and back up one level at a time.
            s.Pad.Up();
            Assert.Equal("Hacks", s.Game);
            s.Pad.A();
            Assert.Equal(["Mario"], Listed(s));
            s.Pad.A();
            Assert.Equal("Hacks/Mario", s.Themed.CurrentFolder);
            Assert.Equal([DeepHack], Listed(s));
            s.Pad.B();
            Assert.Equal(("Hacks", "Mario"), (s.Themed.CurrentFolder, s.Game));
            s.Pad.B();
            Assert.Equal(("", "Hacks"), (s.Themed.CurrentFolder, s.Game));
            s.Pad.B();
            Assert.Equal("system", s.View);
            Assert.Equal("nes", s.System);
        });

        private static string? HelpLabel(ThemedSession s, string entry)
        {
            HintBar bar = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().Single();
            return bar.Entries?.FirstOrDefault(h => h.Button == (entry == "a" ? PadGlyphButton.South : PadGlyphButton.North))?.Label;
        }

        [Fact]
        public Task GB_shows_its_letter_folders_and_the_system_view_counts_every_game_inside_them() => Run(s =>
        {
            s.Pad.Right();
            Assert.Equal("gb", s.System);
            SceneSystem gb = s.Themed.Stage!.Current.Data.System;
            Assert.Equal(3, gb.Counted!.Count);
            Assert.Equal(["A", "B", ThemedSession.GbGames[0]], gb.Games.Select(g => g.Name));
            s.Pad.A();
            Assert.Equal(["A", "B", ThemedSession.GbGames[0]], Listed(s));
            s.Pad.Down();
            s.Pad.A();
            Assert.Equal([BirchEcho], Listed(s));
        });

        // The system view's counter counts every game of a foldered system, not the entries its list opens on.
        [Fact]
        public Task The_system_view_counts_the_games_inside_folders_not_the_entries() => UiTest.Run(() =>
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><text name=\"x\"><systemdata>gamecount</systemdata></text></view>");
            var nes = new ThemeSystem("nes", "Nintendo Entertainment System", "nes");
            var entries = new[] { new SceneGame("USA", "/r/NES/USA") { Folder = true, FolderPath = "USA" }, new SceneGame("Top", "/r/NES/Top.nes") };
            var all = new[] { new SceneGame("A", "/r/NES/USA/A.nes") { FolderPath = "USA", Favorite = true }, new SceneGame("B", "/r/NES/USA/B.nes") { FolderPath = "USA" }, entries[1] };
            var choices = new ThemeChoices { ScreenWidth = 320, ScreenHeight = 200 };
            var data = new SceneData([new SceneSystem(nes, theme.Load(choices, nes), entries) { Counted = all }], new Avalonia.Size(320, 200));
            SceneBuilder scene = SceneBuilder.Build(data.System.Theme.View("system"), data);
            Assert.Equal("3 games available, 1 favorite", ((FontText)scene.Entries.Single(e => e.Element.Name == "x").Control!).Text);
        });

        // USERGUIDE: "folders can't be part of collections", so all games lists the files inside folders, flat.
        [Fact]
        public Task All_games_lists_the_games_inside_folders_flat() => Run(s =>
        {
            for (int guard = 0; guard < 12 && s.System != "all"; guard++) s.Pad.Right();
            s.Pad.A();
            Assert.Equal(16, Listed(s).Length);
            Assert.Contains(DeepHack, Listed(s));
            Assert.All(Entries(s), g => Assert.False(g.Folder));
            Assert.False(s.Themed.InFolder);
        }, ThemedCollectionsTests.AllAuto);

        // P109's return: a game started inside a folder comes back to that folder and game, its first frame a fresh build of that selection (P40's test).
        [Fact]
        public Task A_game_started_inside_a_folder_comes_back_to_that_folder_and_game() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(roms: Library);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal("Racing", Listed(s)[0]);
            s.Pad.A();
            s.Pad.Down();
            Assert.Equal(VelvetRally, s.Game);
            s.Run(1000);
            s.Pad.A();
            Assert.Equal(Path.Combine(s.RomDirectory, "SNES", "Racing", VelvetRally + ".sfc"), Running(s.Window));
            ThemedLibraryFlowTests.Choose(s, "Game Library");
            Assert.Equal(("snes", "Racing", VelvetRally), (s.System, s.Themed.CurrentFolder, s.Game));
            RenderedFrame back = s.Capture();
            SceneData data = s.Themed.Stage!.Current.Data;
            RenderedFrame fresh = SceneAssets.Render(SceneBuilder.Build(data.System.Theme.View("gamelist"), data));
            Assert.Equal(0, SceneAssets.Differing(back, fresh));
            s.Pad.B();
            Assert.Equal(("", "Racing"), (s.Themed.CurrentFolder, s.Game));
            Stop(s.Window);
        }, default);

        [Fact]
        public Task Quick_system_select_keeps_each_system_s_folder_and_game() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "nes");
            s.Pad.Down(2);
            s.Pad.A();
            s.Pad.Down();
            s.Pad.Right();
            Assert.Equal(("gb", ""), (s.System, s.Themed.CurrentFolder));
            s.Pad.A();
            Assert.Equal("A", s.Themed.CurrentFolder);
            s.Pad.Left();
            Assert.Equal(("nes", "USA", UsaGame), (s.System, s.Themed.CurrentFolder, s.Game));
            s.Pad.Right();
            Assert.Equal(("gb", "A", AmberPulse), (s.System, s.Themed.CurrentFolder, s.Game));
        });

        [Fact]
        public Task Sorting_filters_the_search_and_the_jump_behave_within_folders_as_USERGUIDE_describes() => Run(s =>
        {
            ThemedCollectionsTests.Records(s).ToggleFavourite(Path.Combine(s.RomDirectory, "NES", "USA", AmberVale + ".nes"));
            ThemedCollectionsTests.Refresh(s);
            ThemedLibraryPadTests.Enter(s, "nes");

            // "Jump to": the folders sorted on top have one entry, the rest their letters; a list of folders alone is indexed by letter.
            Assert.Equal([GamelistOptions.FolderEntry, "F", "G"], s.Themed.Letters().Select(l => l.Label));
            // "The filters are always applied for the complete game system, including all folder content": their values come from every file.
            Assert.Equal(6, s.Themed.Unfiltered.Count);
            Assert.Equal(["No", "Yes"], GamelistOptions.Values(s.Themed.Unfiltered, FilterField.Favorite));

            // One sort for the whole system: folders on top in its order, games after them in it.
            Now(s, now => s.Themed.ApplyOptions(new GameSort(GameSortKey.Name, true), GameFilter.None, null, now));
            Assert.Equal(["USA", "Hacks", "Europe", ThemedSession.NesGames[1], ThemedSession.NesGames[0]], Listed(s));
            Assert.Equal("Europe", s.Game);
            s.Pad.Up(2);
            s.Pad.A();
            // Favourites first among the folder's games, then the system's order.
            Assert.Equal([AmberVale, UsaGame], Listed(s));
            Assert.Equal([GamelistOptions.Star, "T"], s.Themed.Letters().Select(l => l.Label));
            s.Pad.B();
            s.Pad.Down();
            s.Pad.A();
            Assert.Equal(["M"], s.Themed.Letters().Select(l => l.Label));
            s.Pad.B();

            // "Any folder that contains a favorite game will be displayed", and the others hidden.
            var favourites = GameFilter.None.With(FilterField.Favorite, new HashSet<string> { "Yes" });
            Now(s, now => s.Themed.ApplyOptions(GameSort.Default, favourites, null, now));
            Assert.Equal(["USA"], Listed(s));
            s.Pad.A();
            Assert.Equal([AmberVale], Listed(s));

            // A search that empties the folder shown goes up to the nearest level that still holds a match.
            Now(s, now => s.Themed.ApplyOptions(GameSort.Default, GameFilter.None, null, now));
            Now(s, now => s.Themed.SetFilter("Quartz", now));
            Assert.Equal("", s.Themed.CurrentFolder);
            Assert.Equal(["Europe"], Listed(s));
            Assert.Single(s.Themed.Stage!.Current.Data.System.Counted!);
        });

        [Fact]
        public Task Folders_are_not_on_top_when_the_setting_is_off_and_a_flattened_console_lists_all_its_games() => Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "nes");
            Assert.Equal(["Europe", ThemedSession.NesGames[0], ThemedSession.NesGames[1], "Hacks", "USA"], Listed(s));
            Assert.Equal(["E", "F", "G", "H", "U"], s.Themed.Letters().Select(l => l.Label));
            s.Pad.B();
            s.Pad.Right();
            s.Pad.A();
            Assert.Equal([AmberPulse, BirchEcho, ThemedSession.GbGames[0]], Listed(s));
            Assert.All(Entries(s), g => Assert.False(g.Folder));
            Assert.False(s.Themed.InFolder);
        }, a =>
        {
            a.BigPictureCollections.FoldersOnTop = false;
            a.BigPictureCollections.FlattenedSystems = ["gb"];
        });

        [Fact]
        public Task The_settings_sheet_flattens_a_console_at_once_and_keeps_it_in_appsettings() => Run(s =>
        {
            ThemedCollectionsTests.Choose(s, "Game Collection Settings");
            LunaSwitch flatten = ThemedCollectionsTests.Named<LunaSwitch>(s, "Flatten_nes");
            Assert.False(flatten.IsChecked);
            flatten.IsChecked = true;
            s.Settle();
            Assert.Equal(["nes"], AppSettings.Load().BigPictureCollections.FlattenedSystems);
            Assert.Equal(6, s.Themed.Stage!.Current.Data.Systems.Single(x => x.System.Name == "nes").Games.Count);
            Assert.All(s.Themed.Stage!.Current.Data.Systems.Single(x => x.System.Name == "nes").Games, g => Assert.False(g.Folder));

            LunaSwitch onTop = ThemedCollectionsTests.Named<LunaSwitch>(s, "FoldersOnTop");
            Assert.True(onTop.IsChecked);
            onTop.IsChecked = false;
            s.Settle();
            Assert.False(AppSettings.Load().BigPictureCollections.FoldersOnTop);
            Assert.False(s.Themed.FoldersOnTop);
        });

        // USERGUIDE's "Folder link": A launches the linked file, the menu's Enter Folder overrides it, and editing a collection disables it.
        [Fact]
        public Task A_folder_link_set_in_the_folder_s_editor_launches_its_game_and_Enter_Folder_still_opens_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(roms: Library);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal("Racing", s.Game);
            ThemedCollectionsTests.OpenMenu(s);
            var menu = (GameOptionsWindow)ThemedCollectionsTests.Sheets(s).Current!;
            Assert.DoesNotContain(menu.Options, o => o.Label is "Enter Folder" or "Edit This Game's Metadata" or "Add to Favourites");
            Assert.Contains(menu.Options, o => o.Label == "Edit This Folder's Metadata");
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GameOption_EditThisFoldersMetadata" });
            s.Pad.A();
            s.Settle();
            var editor = Assert.IsType<FolderEditorWindow>(ThemedCollectionsTests.Sheets(s).Current);
            Assert.Empty(s.Window.OwnedWindows);
            Assert.Equal([FolderEditorWindow.NoLink, NeonLoop + ".sfc", VelvetRally + ".sfc"], editor.Link.ItemsSource!.Cast<string>());
            HashSet<InputElement> reached = PadAudit.Reachable(ThemedCollectionsTests.Sheet(s), s.Pad);
            Assert.Empty(PadAudit.Operable(ThemedCollectionsTests.Sheet(s)).Where(c => !reached.Contains(c)).Select(PadAudit.Describe));
            editor.Link.SelectedItem = VelvetRally + ".sfc";
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "FolderSave" });
            s.Pad.A();
            s.Settle();

            string racing = Path.Combine(s.RomDirectory, "SNES", "Racing"), velvet = Path.Combine(racing, VelvetRally + ".sfc");
            Assert.Equal(VelvetRally + ".sfc", ThemedCollectionsTests.Records(s).Edits(racing)[EmuSen.Mistress.Library.GameMetadata.FolderLink]);
            Assert.Equal(velvet, s.Themed.SelectedGame!.FolderLink);
            Assert.Equal("Launch", HelpLabel(s, "a"));

            s.Sounds.Clear();
            s.Pad.A();
            Assert.Equal(["launch"], s.Sounds);
            Assert.Equal(velvet, Running(s.Window));
            ThemedLibraryFlowTests.Choose(s, "Game Library");
            Assert.Equal(("", "Racing"), (s.Themed.CurrentFolder, s.Game));

            ThemedCollectionsTests.OpenMenu(s);
            ThemedCollectionsTests.Reach(s, e => e is Button { Name: "GameOption_EnterFolder" });
            s.Pad.A();
            s.Settle();
            Assert.Equal("Racing", s.Themed.CurrentFolder);
            Assert.Equal([NeonLoop, VelvetRally], Listed(s));
            s.Pad.B();

            // While a custom collection is edited, A enters a linked folder as if it had no link.
            s.Themed.Editing = new EditedCollection(1, "Platform", new HashSet<string>());
            Now(s, now => s.Themed.Refresh(now));
            Assert.Equal("Select", HelpLabel(s, "a"));
            s.Pad.A();
            Assert.Equal("Racing", s.Themed.CurrentFolder);
            Stop(s.Window);
        }, default);

        // A link to a file that has gone is no link: A enters the folder and the help bar says so.
        [Fact]
        public Task A_folder_link_to_a_file_that_has_gone_is_ignored() => Run(s =>
        {
            string usa = Path.Combine(s.RomDirectory, "NES", "USA");
            ThemedCollectionsTests.Records(s).SaveEdits(usa, new Dictionary<string, string?> { [EmuSen.Mistress.Library.GameMetadata.FolderLink] = "Gone (Synthetic).nes" }, DateTime.Now);
            ThemedCollectionsTests.Refresh(s);
            ThemedLibraryPadTests.Enter(s, "nes");
            s.Pad.Down(2);
            Assert.Equal("USA", s.Game);
            Assert.Null(s.Themed.SelectedGame!.FolderLink);
            Assert.Equal("Select", HelpLabel(s, "a"));
            s.Pad.A();
            Assert.Equal("USA", s.Themed.CurrentFolder);
        });

        // THEMES.md "defaultFolderImage" and USERGUIDE's media path: a folder's own picture is <system>/<type>/<parent folders>/<folder name>.
        [Fact]
        public void A_folder_s_picture_is_named_after_it_and_presence_from_one_listing_equals_presence_asked_game_by_game()
        {
            string root = Path.Combine(Path.GetTempPath(), "EmuSenFolderMedia", Guid.NewGuid().ToString("N"));
            try
            {
                var media = new EsdeMediaFolder(root);
                var nes = new ThemeSystem("nes", "Nintendo Entertainment System", "nes");
                string usa = Touch(Path.Combine(root, "nes", "covers", "USA.png"));
                string mario = Touch(Path.Combine(root, "nes", "screenshots", "Hacks", "Mario.jpg"));
                Assert.Equal(usa, media.Find(nes, new SceneGame("USA", "/r/NES/USA") { Folder = true, FolderPath = "USA" }, "cover"));
                Assert.Equal(mario, media.Find(nes, new SceneGame("Mario", "/r/NES/Hacks/Mario") { Folder = true, FolderPath = "Hacks/Mario" }, "screenshot"));
                Assert.Null(media.Find(nes, new SceneGame("Mario", "/r/NES/Hacks/Mario") { Folder = true, FolderPath = "Hacks/Mario" }, "cover"));

                // A foldered game's own file first, then the flat name ES-DE gives a flattened system's games (Q61).
                string own = Touch(Path.Combine(root, "nes", "marquees", "USA", "Game.png"));
                string flat = Touch(Path.Combine(root, "nes", "marquees", "Game.png"));
                var game = new SceneGame("Game", "/r/NES/USA/Game.nes") { FolderPath = "USA" };
                Assert.Equal(own, media.Find(nes, game, "marquee"));
                File.Delete(own);
                Assert.Equal(flat, media.Find(nes, game, "marquee"));
                Assert.Equal(flat, media.Find(nes, game with { FolderPath = "" }, "marquee"));
                Touch(own);

                // A stray in another folder names no game here, and a picture in a subfolder counts once a game is in it.
                Touch(Path.Combine(root, "nes", "titlescreens", "Europe", "Game.png"));
                Touch(Path.Combine(root, "nes", "fanart", "USA", "Other.png"));
                var games = new[] { game, new SceneGame("USA", "/r/NES/USA") { Folder = true, FolderPath = "USA" }, new SceneGame("Top", "/r/NES/Top.nes") };
                IReadOnlySet<string> listed = media.Present(nes, games);
                var asked = ThemeCapabilities.MediaTypes.Where(t => games.Any(g => media.Find(nes, g, t) is not null)).ToHashSet();
                Assert.Equal(asked.Order(), listed.Order());
                Assert.Equal(["cover", "marquee"], listed.Order());

                // A type whose only file is in a type folder's own folder counts too.
                Touch(Path.Combine(root, "nes", "screenshots", "USA", "Game.png"));
                Assert.Equal(["cover", "marquee", "screenshot"], media.Present(nes, games).Order());

                // A file added to a type folder's own folder changes the stamp.
                string before = media.Stamp(nes)!;
                System.Threading.Thread.Sleep(20);
                Touch(Path.Combine(root, "nes", "screenshots", "Hacks", "Added.png"));
                Directory.SetLastWriteTimeUtc(Path.Combine(root, "nes", "screenshots", "Hacks"), DateTime.UtcNow.AddMinutes(1));
                Assert.NotEqual(before, media.Stamp(nes));
            }
            finally
            {
                try { Directory.Delete(root, recursive: true); } catch { }
            }
        }

        private static MediaSources Sources(ThemedSession s) => (MediaSources)typeof(MainWindow).GetMethod("MediaSourcesNow", Hidden)!.Invoke(s.Window, null)!;

        private static string Touch(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, [0x89, 0x50, 0x4E, 0x47]);
            return path;
        }

        // P108, shown on the unchanged reader (5175e909, 0 of 6 found) and fixed - see EmuSen_BigPicture.md §30.1.
        [Fact]
        public Task P108_media_of_a_game_in_a_folder_are_found_under_that_folder_in_an_ES_DE_tree_and_in_Mistress_s_store() => Session.Dispatch(() =>
        {
            string esde = Path.Combine(Path.GetTempPath(), "EmuSenP108", Guid.NewGuid().ToString("N"));
            try
            {
                using var s = new ThemedSession(settings: a => a.EsdeMediaDirectory = esde, roms: RegionFolders);
                string usa = Path.Combine(s.RomDirectory, "NES", "USA", UsaGame + ".nes"), europe = Path.Combine(s.RomDirectory, "NES", "Europe", EuropeGame + ".nes");
                var expected = new Dictionary<(string Rom, string Type), string>
                {
                    [(usa, "cover")] = Touch(Path.Combine(esde, "nes", "covers", "USA", UsaGame + ".png")),
                    [(usa, "screenshot")] = Touch(Path.Combine(esde, "nes", "screenshots", "USA", UsaGame + ".jpg")),
                    [(usa, "marquee")] = Touch(Path.Combine(esde, "nes", "marquees", "USA", UsaGame + ".webp")),
                    [(usa, "video")] = Touch(Path.Combine(esde, "nes", "videos", "USA", UsaGame + ".mp4")),
                    [(europe, "cover")] = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "covers", "Europe", EuropeGame + ".png")),
                    [(europe, "screenshot")] = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "screenshots", "Europe", EuropeGame + ".png")),
                };

                MediaSources sources = Sources(s);
                var found = expected.ToDictionary(p => p.Key, p => sources.Locate("nes", p.Key.Rom, p.Key.Type).Path);
                foreach (var p in found) _out.WriteLine($"{Path.GetFileName(p.Key.Rom)} {p.Key.Type}: {p.Value ?? "(none)"}");
                Assert.Equal(expected.Count, found.Count(p => p.Value == expected[p.Key]));
                Assert.Equal(MediaSource.EsdeFolder, sources.Locate("nes", usa, "cover").Source);
                Assert.Equal(MediaSource.ScreenScraper, sources.Locate("nes", europe, "cover").Source);
            }
            finally
            {
                try { Directory.Delete(esde, recursive: true); } catch { }
            }
        }, default);

        // A store an earlier build wrote flat is put in folders when Mistress opens it, and the game shows its picture from there.
        [Fact]
        public Task Mistress_puts_its_own_store_in_folders_when_it_opens_it_and_the_picture_is_still_found() => Session.Dispatch(() =>
        {
            string flat = "";
            using var s = new ThemedSession(roms: roms =>
            {
                RegionFolders(roms);
                using MediaStore store = MediaStore.Open(MediaStore.DefaultRoot);
                string rom = Path.Combine(roms, "NES", "USA", UsaGame + ".nes");
                store.RememberFile(rom, 64, 1, "u");
                store.Record(new ScrapedRecord("u", 64, ScrapeState.Found) { FetchedAt = DateTime.UtcNow });
                store.RecordMedia("u", 64, new StoredMedia("cover", Path.Combine("nes", "covers", UsaGame + ".png"), "us", null), DateTimeOffset.UtcNow);
                flat = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "covers", UsaGame + ".png"));
            });
            string foldered = Path.Combine(MediaStore.DefaultRoot, "nes", "covers", "USA", UsaGame + ".png");
            Assert.True(File.Exists(foldered));
            Assert.False(File.Exists(flat));
            Assert.Equal(foldered, Sources(s).Cover("nes", Path.Combine(s.RomDirectory, "NES", "USA", UsaGame + ".nes")));

            // Clear takes the game's pictures from its own folder, a file media.db does not name among them, and nothing of another folder's game.
            string named = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "screenshots", "USA", UsaGame + ".png"));
            string other = Touch(Path.Combine(MediaStore.DefaultRoot, "nes", "covers", "Europe", EuropeGame + ".png"));
            s.Window.ClearMetadata(Path.Combine(s.RomDirectory, "NES", "USA", UsaGame + ".nes"));
            Assert.False(File.Exists(foldered));
            Assert.False(File.Exists(named));
            Assert.True(File.Exists(other));
        }, default);
    }
}
