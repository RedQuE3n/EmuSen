using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // The Home card's own behaviour: Continue and its pane, the game in hand with Y, the tools' lines, Power in each session - see EmuSen_BigPicture.md §49.
    [Collection(TestCollections.ProcessGlobals)]
    public class HomeCardTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(HomeCardTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private const string Game = "Cobalt Harbor (Synthetic)";
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenHomeCardTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public HomeCardTests()
        {
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
        }

        public void Dispose()
        {
            ToolCounts.Quiet();
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A game running in a big-screen session, or on the desktop, then left for the library with the game kept.
        private (MainWindow Window, PadDriver Pad) Suspended(bool bigScreen = false, bool gameMode = false)
        {
            File.WriteAllBytes(Path.Combine(_romDir, Game + ".sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen, StateDirectory = Path.Combine(_root, "States") };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            if (gameMode) typeof(MainWindow).GetField("_bigScreenForced", Hidden)!.SetValue(window, true);
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", Hidden)!.Invoke(window, null);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            typeof(MainWindow).GetMethod("ToggleLibrary", Hidden)!.Invoke(window, null);
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);
            return (window, pad);
        }

        private static void Done(MainWindow window)
        {
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            window.Close();
        }

        private static T Named<T>(Control root, string name) where T : Control => root.GetLogicalDescendants().OfType<T>().Single(c => c.Name == name);

        private static GameRecords Records(MainWindow w) => (GameRecords)typeof(MainWindow).GetField("_records", Hidden)!.GetValue(w)!;

        private static string RomPath(MainWindow w) => (string)typeof(MainWindow).GetField("_currentRomPath", Hidden)!.GetValue(w)!;

        [Fact]
        public Task Continue_shows_the_paused_frame_the_name_and_the_time_and_A_resumes_the_game() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Suspended();
            pad.Start();
            Assert.Equal(MainWindow.ContinueRow, PadMenu.SelectedLine(window));
            PauseCardPane pane = PadMenu.Pane(window);
            Assert.Equal("frame", pane.Showing);
            Assert.NotNull(Named<Image>(pane, "PanePicture"));
            Assert.Equal(Game, PadMenu.Entries(window)[0].Value!());
            Assert.EndsWith(" · Paused just now", Named<TextBlock>(pane, "PaneCaption").Text);
            Assert.False(window.GetControl<Control>("MenuBackdrop").IsVisible);

            pad.A();
            Assert.False(PadMenu.IsOpen(window));
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            Done(window);
        }, default);

        [Theory]
        [InlineData(0, "Paused just now")]
        [InlineData(12, "Paused 12 min ago")]
        [InlineData(150, "Paused 2 h ago")]
        [InlineData(60 * 50, "Paused 2 d ago")]
        public void Paused_reads_minutes_then_hours_then_days(int minutes, string expected) =>
            Assert.Equal(expected, typeof(MainWindow).GetMethod("PausedAgo", BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, [TimeSpan.FromMinutes(minutes) + TimeSpan.FromSeconds(1)]));

        [Fact]
        public Task The_game_in_hand_shows_its_name_and_console_and_Y_makes_it_a_favourite_where_it_is_in_hand() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Suspended();
            pad.Start();
            pad.Down();
            Assert.Equal(Game, PadMenu.SelectedLine(window));
            Assert.Equal("frame", PadMenu.Pane(window).Showing);
            Assert.Equal(Game, Named<TextBlock>(PadMenu.Pane(window), "PaneTitle").Text);
            Assert.Contains("Favourite", PadMenu.Big(window).Hints!.Select(h => h.Label));

            string rom = RomPath(window);
            Assert.False(Records(window).IsFavourite(rom));
            pad.Y();
            Assert.True(Records(window).IsFavourite(rom));
            Assert.True(PadMenu.IsOpen(window));

            Assert.EndsWith("· Favourite", Named<TextBlock>(PadMenu.Pane(window), "PaneCaption").Text);
            // A on the game opens its page, where Y is the favourite too (EmuSen_BigPicture.md §50.1).
            pad.A();
            Assert.False(PadMenu.IsOpen(window));
            Assert.Equal(rom, window.GamePageShown?.Path);
            Assert.Contains("Favourite", window.GamePageLayout!.Hints.Select(h => h.Label));
            pad.Y();
            Assert.False(Records(window).IsFavourite(rom));

            // Off the game, Y does nothing and the help bar does not offer it.
            pad.B();
            Assert.Null(window.GamePageShown);
            pad.Start();
            PadMenu.Choose(window, pad, MainWindow.ToolsMenu, exact: true);
            Assert.DoesNotContain("Favourite", PadMenu.Big(window).Hints!.Select(h => h.Label));
            pad.Y();
            Assert.False(Records(window).IsFavourite(rom));
            Done(window);
        }, default);

        [Fact]
        public Task The_tools_row_lists_each_tool_s_line_and_a_tool_s_row_shows_its_own() => Session.Dispatch(() =>
        {
            ToolCounts.Answer();
            (MainWindow window, PadDriver pad) = Suspended();
            pad.Start();
            pad.Down(PadMenu.PathTo(window, MainWindow.ToolsMenu, exact: true)![0]);
            PauseCardPane pane = PadMenu.Pane(window);
            Assert.Equal("lines", pane.Showing);
            string[] words = pane.GetLogicalDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
            Assert.Equal([MainWindow.ScraperRow, "41 without art", MainWindow.PackagesMenu, "3 changed", MainWindow.LibraryManagerMenu, "⚠ 2 suspect dumps", MainWindow.CoresMenu, "⚠ 1 to approve", MainWindow.CheatsRow, "12 on · 3 games"], words);

            pad.A();
            pad.Down(3);
            Assert.Equal(MainWindow.CoresMenu, PadMenu.SelectedLine(window));
            words = pane.GetLogicalDescendants().OfType<TextBlock>().Select(t => t.Text ?? "").ToArray();
            Assert.Equal([MainWindow.CoresMenu, "⚠ 1 to approve"], words);
            Assert.Equal("⚠ 1 to approve", PadMenu.Entries(window)[3].Value?.Invoke());
            Assert.Equal("41 no art", PadMenu.Entries(window)[0].Value?.Invoke());
            pad.Down(2);
            Assert.Equal("none", pane.Showing);
            Done(window);
        }, default);

        [Theory]
        [InlineData(true, false, new[] { "Exit Big Picture", MainWindow.QuitEmuSenRow })]
        [InlineData(true, true, new[] { MainWindow.QuitEmuSenRow })]
        [InlineData(false, false, new[] { "Full Screen", "Big Picture", MainWindow.QuitEmuSenRow })]
        public Task Power_holds_exit_big_picture_and_quit_and_in_game_mode_quit_alone(bool bigScreen, bool gameMode, string[] rows) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Suspended(bigScreen, gameMode);
            if (bigScreen)
            {
                // On the big screen's own look Power is a place on Home, which opens the card at its page (EmuSen_BigPicture.md §51.2).
                window.HomeLayout!.FocusOn(HubRow.Places, "power");
                pad.A();
            }
            else
            {
                pad.Start();
                Assert.True(PadMenu.Card(window).IsVisible);
                PadMenu.Choose(window, pad, MainWindow.PowerMenu, exact: true);
            }
            Assert.Equal(rows, PadMenu.Lines(window));
            PadMenu.Choose(window, pad, MainWindow.QuitEmuSenRow, exact: true);
            Assert.Equal("The game closes, then EmuSen.", PadMenu.Big(window).Subtitle);
            PadMenu.Answer(window, pad, yes: false);
            Assert.Equal(bigScreen ? 1 : 2, PadMenu.Depth(window));
            Done(window);
        }, default);
    }
}
