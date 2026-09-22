using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
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
