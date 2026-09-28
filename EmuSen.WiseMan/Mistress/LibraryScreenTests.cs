using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The OpenEmu-shaped library screen: the sidebar, the covers, favourites and the media views - see EmuSen_Settings_Reference.md §4.33 to §4.35.
    [Collection(TestCollections.ProcessGlobals)]
    public class LibraryScreenTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(LibraryScreenTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;
        private readonly string _states;

        public LibraryScreenTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenLibraryScreenTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            _states = Path.Combine(_root, "States");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void Rom(string name) => File.WriteAllBytes(Path.Combine(_romDir, name), SyntheticRom.BuildBlank());

        private MainWindow Open(string view = AppSettings.LibraryGrid)
        {
            new AppSettings { RomDirectory = _romDir, StateDirectory = _states, LibraryView = view, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1024, Height = 768 };
            window.Show();
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private static TileGrid<RomEntry> Grid(MainWindow w) => w.GetControl<TileGrid<RomEntry>>("LibraryGrid");
        private static TileGrid<MediaItem> Media(MainWindow w) => w.GetControl<TileGrid<MediaItem>>("MediaGrid");
        private static LunaList<RomEntry> List(MainWindow w) => (LunaList<RomEntry>)w.GetControl<ListBox>("LibraryList");
        private static SourceList Sidebar(MainWindow w) => w.GetControl<SourceList>("LibrarySidebar");
        private static string Header(MainWindow w) => w.GetControl<TextBlock>("LibraryHeaderText").Text ?? "";
        private static string[] Shown(MainWindow w) => ((IReadOnlyList<RomEntry>)Field(w, "_shownEntries")).Select(e => e.FileName).ToArray();

        [Fact]
        public Task The_grid_is_the_default_and_the_list_is_one_toggle_away() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.nes");
            MainWindow window = Open();

            Assert.True(Grid(window).IsVisible);
            Assert.False(List(window).IsVisible);

            Invoke(window, "ShowLibraryAs", AppSettings.LibraryList);

            Assert.False(Grid(window).IsVisible);
            Assert.True(List(window).IsVisible);
            Assert.Equal(AppSettings.LibraryList, AppSettings.Load().LibraryView);
            window.Close();
        }, default);

        [Fact]
        public Task A_console_in_the_sidebar_narrows_the_covers_and_the_counts_are_every_console_s() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.nes");
            Rom("Gamma.nes");
            MainWindow window = Open();
            Assert.Equal(3, Shown(window).Length);

            Choose(window, MainWindow.ConsoleKeyPrefix + "NES (Moon)");

            Assert.Equal(new[] { "Beta.nes", "Gamma.nes" }, Shown(window));
            Assert.Equal("NES (Moon)", AppSettings.Load().SelectedCore);

            Choose(window, MainWindow.AllGamesKey);
            Assert.Equal(3, Shown(window).Length);
            window.Close();
        }, default);

        // The status bar and each of its parts follow Preferences at once and are remembered - see EmuSen_Settings_Reference.md §4.51.
        [Fact]
        public Task The_status_bar_and_its_parts_follow_preferences_at_once_and_are_remembered() => Session.Dispatch(() =>
        {
            MainWindow window = Open(AppSettings.LibraryList);
            Control bar = window.GetControl<Border>("StatusBar");
            TextBlock text = window.GetControl<TextBlock>("StatusText"), fps = window.GetControl<TextBlock>("FpsText");
            Assert.True(bar.IsVisible && text.IsVisible && fps.IsVisible);

            Invoke(window, "ShowPreferences");
            Dispatcher.UIThread.RunJobs();
            var preferences = window.OwnedWindows.OfType<PreferencesWindow>().Single();
            LunaSwitch Toggle(string name) => preferences.GetLogicalDescendants().OfType<LunaSwitch>().Single(t => t.Name == name);
            void Set(string name, bool on) { Toggle(name).IsChecked = on; Dispatcher.UIThread.RunJobs(); }

            Set("ShowFpsBarSwitch", false);
            Assert.True(bar.IsVisible && text.IsVisible);
            Assert.False(fps.IsVisible);

            Set("ShowStatusTextSwitch", false);
            Assert.False(bar.IsVisible);

            Set("ShowStatusTextSwitch", true);
            Assert.True(bar.IsVisible && text.IsVisible);

            Set("ShowStatusBarSwitch", false);
            Assert.False(bar.IsVisible);
            AppSettings saved = AppSettings.Load();
            Assert.False(saved.ShowStatusBar);
            Assert.True(saved.ShowStatusText);
            Assert.False(saved.ShowFpsBar);
            preferences.Close();
            window.Close();

            var again = new MainWindow { Width = 1024, Height = 768 };
            again.Show();
            Dispatcher.UIThread.RunJobs();
            Assert.False(again.GetControl<Border>("StatusBar").IsVisible);
            again.Close();
        }, default);

        // The Game Boy's core on two shelves, the Color's listed after it and chosen like any console - see EmuSen_Settings_Reference.md §4.46.
        [Fact]
        public Task Game_Boy_Color_games_have_their_own_row_in_the_sidebar() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            var plain = new byte[0x150];
            File.WriteAllBytes(Path.Combine(_romDir, "Mono.gb"), plain);
            var color = new byte[0x150];
            color[0x143] = 0x80;
            File.WriteAllBytes(Path.Combine(_romDir, "Tinted.gb"), color);
            File.WriteAllBytes(Path.Combine(_romDir, "Crystal.gbc"), plain);
            MainWindow window = Open(AppSettings.LibraryList);

            string[] rows = Sidebar(window).GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
            int gb = Array.IndexOf(rows, "GB"), gbc = Array.IndexOf(rows, "GBC");
            Assert.True(gb >= 0 && gbc > gb, string.Join(" | ", rows));
            Assert.Equal("1", rows[gb + 1]);
            Assert.Equal("2", rows[gbc + 1]);
            UiTest.Dump("library-gbc-sidebar", UiTest.Capture(window));

            Choose(window, MainWindow.ConsoleKeyPrefix + EmuSen.Cores.CoreCatalog.GameBoyColorShelf);
            Assert.Equal(new[] { "Crystal.gbc", "Tinted.gb" }, Shown(window));
            Assert.Equal(EmuSen.Cores.CoreCatalog.GameBoyColorShelf, AppSettings.Load().SelectedCore);

            Choose(window, MainWindow.ConsoleKeyPrefix + "Game Boy (Mercury)");
            Assert.Equal(new[] { "Mono.gb" }, Shown(window));
            window.Close();
        }, default);

        [Fact]
        public Task A_favourite_marked_from_the_pad_is_in_the_favourites_collection_and_nothing_else_is() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.sfc");
            MainWindow window = Open();
            List(window).Select(Entry(window, "Beta.sfc"));

            Pad(window, UiButton.Options);
            Choose(window, MainWindow.FavouritesKey);

            Assert.Equal(new[] { "Beta.sfc" }, Shown(window));
            Assert.Equal(MainWindow.FavouritesKey, Sidebar(window).SelectedKey);

            Pad(window, UiButton.Options);
            Assert.Empty(Shown(window));
            Assert.Contains("No favourites yet", Header(window));
            window.Close();
        }, default);

        [Fact]
        public Task Recently_played_is_the_games_started_newest_first() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.sfc");
            Rom("Gamma.sfc");
            using (GameRecords records = GameRecords.Load())
            {
                records.Started(Path.Combine(_romDir, "Gamma.sfc"), new DateTime(2026, 9, 1));
                records.Started(Path.Combine(_romDir, "Alpha.sfc"), new DateTime(2026, 9, 20));
            }
            MainWindow window = Open();

            Choose(window, MainWindow.RecentKey);

            Assert.Equal(new[] { "Alpha.sfc", "Gamma.sfc" }, Shown(window));
            window.Close();
        }, default);

        [Fact]
        public Task Choosing_a_cover_selects_the_same_game_the_launch_path_reads() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.sfc");
            MainWindow window = Open();
            RomEntry beta = Entry(window, "Beta.sfc");

            List(window).Select(beta);

            Assert.Same(beta, Grid(window).Selected);
            window.Close();
        }, default);

        [Fact]
        public Task On_a_pad_the_covers_move_by_a_row_and_the_shoulders_take_the_console() => Session.Dispatch(() =>
        {
            for (int i = 0; i < 12; i++) Rom($"Game {i:D2}.sfc");
            Rom("Other.nes");
            MainWindow window = Open();
            Dispatcher.UIThread.RunJobs();
            int columns = Grid(window).Columns;
            Assert.True(columns > 1, $"{columns} columns");
            List(window).SelectedIndex = 0;

            Pad(window, UiButton.Down);
            Assert.Equal(columns, List(window).SelectedIndex);
            Pad(window, UiButton.Right);
            Assert.Equal(columns + 1, List(window).SelectedIndex);

            string before = AppSettings.Load().SelectedCore;
            Pad(window, UiButton.PageDown);
            Assert.NotEqual(before, AppSettings.Load().SelectedCore);
            window.Close();
        }, default);

        [Fact]
        public Task The_save_states_view_shows_each_state_under_its_game_and_the_sidebar_narrows_it() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            Rom("Beta.nes");
            Directory.CreateDirectory(_states);
            File.WriteAllBytes(Path.Combine(_states, "Alpha.state"), new byte[] { 1 });
            File.WriteAllBytes(Path.Combine(_states, "Alpha.resume.state"), new byte[] { 1 });
            File.WriteAllBytes(Path.Combine(_states, "Beta.slot3.state"), new byte[] { 1 });
            MainWindow window = Open();

            Invoke(window, "ShowCategory", MainWindow.StatesCategory);

            Assert.True(Media(window).IsVisible);
            Assert.False(Grid(window).IsVisible);
            Assert.Equal(3, MediaShown(window).Length);
            Assert.Contains("3 save states", Header(window));

            Choose(window, MainWindow.ConsoleKeyPrefix + "NES (Moon)");
            MediaItem beta = Assert.Single(MediaShown(window));
            Assert.Equal("Slot 3", beta.Label);

            Invoke(window, "ShowCategory", MainWindow.LibraryCategory);
            Assert.True(Grid(window).IsVisible);
            window.Close();
        }, default);

        [Fact]
        public Task Big_screen_hides_the_sidebar_and_gives_the_console_back_to_the_filter_bar() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            new AppSettings { RomDirectory = _romDir, BigScreen = true }.Save();
            var window = new MainWindow();
            window.Show();

            Assert.False(window.GetControl<Control>("LibrarySidebarPane").IsVisible);
            Assert.True(window.GetControl<FilterBar>("LibraryFilter").ShowFacet);
            window.Close();
        }, default);

        [Fact]
        public Task A_steam_launch_in_desktop_mode_keeps_the_sidebar_and_the_menu_bar() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            MainWindow window = UnderSteam(new() { ["SteamDeck"] = "1", ["SteamGameId"] = "123", ["XDG_CURRENT_DESKTOP"] = "KDE" });

            Assert.True(window.GetControl<Control>("LibrarySidebarPane").IsVisible);
            Assert.True(window.GetControl<Control>("MenuStrip").IsVisible);
            Assert.False(window.GetControl<FilterBar>("LibraryFilter").ShowFacet);
            window.Close();
        }, default);

        [Fact]
        public Task Game_mode_under_gamescope_still_starts_the_big_screen() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            MainWindow window = UnderSteam(new() { ["SteamDeck"] = "1", ["SteamGameId"] = "123", ["XDG_CURRENT_DESKTOP"] = "gamescope" });

            Assert.False(window.GetControl<Control>("LibrarySidebarPane").IsVisible);
            Assert.False(window.GetControl<Control>("MenuStrip").IsVisible);
            Assert.True(window.GetControl<FilterBar>("LibraryFilter").ShowFacet);
            window.Close();
        }, default);

        [Fact]
        public Task A_deck_that_names_no_desktop_is_taken_for_game_mode() => Session.Dispatch(() =>
        {
            Rom("Alpha.sfc");
            MainWindow window = UnderSteam(new() { ["SteamDeck"] = "1", ["XDG_CURRENT_DESKTOP"] = null });

            Assert.False(window.GetControl<Control>("LibrarySidebarPane").IsVisible);
            window.Close();
        }, default);

        // The window reads the session once, in its constructor, so the variables are restored as soon as it exists.
        private MainWindow UnderSteam(Dictionary<string, string?> environment)
        {
            var saved = environment.Keys.ToDictionary(k => k, Environment.GetEnvironmentVariable);
            foreach (var (key, value) in environment) Environment.SetEnvironmentVariable(key, value);
            try { return Open(); }
            finally { foreach (var (key, value) in saved) Environment.SetEnvironmentVariable(key, value); }
        }

        private static RomEntry Entry(MainWindow w, string name) => ((IReadOnlyList<RomEntry>)Field(w, "_shownEntries")).Single(e => e.FileName == name);

        private static MediaItem[] MediaShown(MainWindow w) => ((IReadOnlyList<MediaItem>)Field(w, "_shownMedia")).ToArray();

        private static void Choose(MainWindow window, string key) => Invoke(window, "ChooseCollection", key);

        private static void Pad(MainWindow window, UiButton button) => Invoke(window, "OnPadCommand", button);

        private static object Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static void Invoke(MainWindow window, string name, params object[] args) =>
            typeof(MainWindow).GetMethod(name, BindingFlags.Instance | BindingFlags.NonPublic, null, Array.ConvertAll(args, a => a.GetType()), null)!.Invoke(window, args);
    }
}
