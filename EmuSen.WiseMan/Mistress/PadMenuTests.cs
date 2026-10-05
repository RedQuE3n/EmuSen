using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // The pad menu's pages: the game's and the main menu's rows, the submenus and B back through them, the slot on the Save and Load rows, the questions, and each row's window - see EmuSen_Settings_Reference.md §4.69.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadMenuTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadMenuTests).GetTypeInfo().Assembly);

        private const BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadMenuTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadMenuTests()
        {
            _romDir = Path.Combine(_root, "Roms");
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

        internal static readonly string[] GameSettings = ["Cheats", "Players & Controllers", "Controller Bindings", "Graphics", "Shaders"];
        internal static readonly string[] Settings = ["Players & Controllers", "Controller Bindings", "Graphics", "Shaders", "Cheats", "Firmware", "Preferences"];

        // One synthetic game, started from the library: as Game Mode starts it with a big screen, else on the desktop.
        private (MainWindow Window, PadDriver Pad) Playing(bool bigScreen, int width = 1280, int height = 800)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Cobalt Harbor (Synthetic).sfc"), PadRewindReelTests.ChangingBackdrop());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen, StateDirectory = Path.Combine(_root, "States") };
            app.BigPictureInterface.MenuOpeningEffect = BigPictureInterface.OpeningNone;
            app.Save();
            var window = new MainWindow { Width = width, Height = height };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            if (bigScreen) pad.A();
            else Call(window, "LaunchSelectedLibraryEntry");
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private static object? Call(MainWindow w, string method, params object[] args) => typeof(MainWindow).GetMethod(method, Hidden | BindingFlags.Public)!.Invoke(w, args);
        private static object? Field(MainWindow w, string name) => typeof(MainWindow).GetField(name, Hidden)!.GetValue(w);
        private static string? Running(MainWindow w) => (string?)Field(w, "_currentRomPath");
        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");
        private static void Chord(PadDriver pad) => pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);

        private static void Done(MainWindow window)
        {
            Call(window, "StopEmulationThread");
            window.Close();
        }

        // Down to a row of the page shown, by its text, and A.
        private static void Enter(MainWindow w, PadDriver pad, string row)
        {
            int at = Array.FindIndex(PadMenu.Lines(w), l => l.StartsWith(row, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{row}' on the page: {string.Join(", ", PadMenu.Lines(w))}");
            pad.Down((at - PadMenu.Selected(w) + PadMenu.Lines(w).Length) % PadMenu.Lines(w).Length);
            pad.A();
        }

        private static string[] SubmenuLines(MainWindow w, string submenu) =>
            PadMenu.Entries(w).Single(e => e.Text() == submenu).Submenu!().Select(e => e.Text()).ToArray();

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task The_in_game_menu_is_the_game_s_actions_then_its_settings_then_leaving(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            Chord(pad);
            string[] lines = PadMenu.Lines(window);
            Assert.Equal(10, lines.Length);
            Assert.Equal("Resume", lines[0]);
            Assert.StartsWith("Rewind", lines[1]);
            Assert.Equal("Save State      <  Slot 1 (Empty)  >", lines[2]);
            Assert.Equal("Load State      <  Slot 1 (Empty)  >", lines[3]);
            Assert.Equal("Speed      <  100%  >", lines[4]);
            Assert.Equal(["Restart Game...", MainWindow.GameSettingsMenu, "Back to Library", "Quit Game...", MainWindow.EmuSenMenu], lines[5..]);
            Assert.DoesNotContain(lines, l => l.StartsWith("State Slot", StringComparison.Ordinal));

            // Two dividers: before the settings, and before the ways out.
            List<PadMenuEntry> rows = PadMenu.Entries(window);
            Assert.Equal([6, 7], Enumerable.Range(0, rows.Count).Where(i => rows[i].StartsSection));
            Assert.All(new[] { 2, 3, 4 }, i => Assert.NotNull(rows[i].Adjust));
            Assert.Equal(["Save State", "Load State"], rows.Where(r => r.ShowsSlot).Select(r => r.Label!()));
            Assert.Equal(["Restart Game?", "Quit Game?"], rows.Where(r => r.Question is not null).Select(r => r.Question!));

            Assert.Equal(GameSettings, SubmenuLines(window, MainWindow.GameSettingsMenu));
            Assert.Equal(bigScreen ? ["Preferences", "Exit Big Picture", "Exit EmuSen..."] : ["Preferences", "Full Screen", "Big Picture", "Exit EmuSen..."], SubmenuLines(window, MainWindow.EmuSenMenu));

            // The big panel: the game's name over it, a chevron on each submenu, arrows on each value, no version under the first page.
            if (bigScreen)
            {
                MenuPanel big = PadMenu.Big(window);
                Assert.Equal("Cobalt Harbor (Synthetic)", big.Title);
                Assert.Null(big.Subtitle);
                Assert.Null(big.Footer);
                Assert.Equal(["Close Menu", "Select", "Close Menu", "Change", "Choose"], big.Hints!.Select(h => h.Label));
            }
            else Assert.Equal("Cobalt Harbor (Synthetic)", PadMenu.DeskTitle(window));
            Done(window);
        }, default);

        [Fact]
        public Task The_main_menu_is_the_game_s_entries_then_library_settings_and_emusen() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: false);
            Call(window, "ToggleLibrary");
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.Start();
            Assert.Equal(["Back to Cobalt Harbor (Synthetic)", "Game Options...", "Scrape This Game...", MainWindow.LibraryMenu, MainWindow.SettingsMenu, MainWindow.EmuSenMenu], PadMenu.Lines(window));
            Assert.Equal([3], Enumerable.Range(0, 6).Where(i => PadMenu.Entries(window)[i].StartsSection));
            Assert.Equal(["Scrape Games..."], SubmenuLines(window, MainWindow.LibraryMenu));
            Assert.Equal(Settings, SubmenuLines(window, MainWindow.SettingsMenu));
            Assert.Equal(["Full Screen", "Big Picture", "Exit EmuSen..."], SubmenuLines(window, MainWindow.EmuSenMenu));
            Assert.Equal(MainWindow.MainMenuTitle, PadMenu.DeskTitle(window));
            Done(window);
        }, default);

        // Over the themed view with no game: no divider above the first row, and the library's own entries in Library.
        [Fact]
        public Task The_themed_main_menu_folds_collections_and_theme_settings_into_library() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            Assert.Equal(["Scrape This Game...", MainWindow.LibraryMenu, MainWindow.SettingsMenu, MainWindow.EmuSenMenu], PadMenu.Lines(s.Window));
            Assert.Equal(["Collections", "Scrape Games...", "Theme Settings"], SubmenuLines(s.Window, MainWindow.LibraryMenu));
            Assert.Equal(["Exit Big Picture", "Exit EmuSen..."], SubmenuLines(s.Window, MainWindow.EmuSenMenu));
            Assert.Equal(MainWindow.MainMenuTitle, PadMenu.Big(s.Window).Title);

            s.Pad.Up();
            Assert.Equal(MainWindow.EmuSenMenu, PadMenu.SelectedLine(s.Window));
            s.Pad.A();
            Assert.Equal(MainWindow.EmuSenMenu, PadMenu.Big(s.Window).Title);
            Assert.StartsWith("EmuSen 0.", PadMenu.Big(s.Window).Footer);
            Assert.Equal("‹ MAIN MENU", PadMenu.Big(s.Window).Subtitle);
            Assert.Equal("Back", PadMenu.Big(s.Window).Hints![2].Label);
        }, default);

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task A_submenu_is_entered_with_A_left_with_B_to_the_row_it_was_entered_from_and_remembers_its_row(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            Chord(pad);
            Assert.True(window.IsPaused);
            Enter(window, pad, MainWindow.GameSettingsMenu);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal(GameSettings, PadMenu.Lines(window));
            Assert.Equal(0, PadMenu.Selected(window));
            if (bigScreen)
            {
                Assert.Equal(MainWindow.GameSettingsMenu, PadMenu.Big(window).Title);
                Assert.Equal("‹ COBALT HARBOR (SYNTHETIC)", PadMenu.Big(window).Subtitle);
            }
            else Assert.Equal("Cobalt Harbor (Synthetic)  ›  Game Settings", PadMenu.DeskTitle(window));

            pad.Down(2);
            pad.B();
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Equal(MainWindow.GameSettingsMenu, PadMenu.SelectedLine(window));
            Assert.True(window.IsPaused);

            // Entered again it opens on the row it was left on; Start closes the whole menu from inside it, and the game runs.
            pad.A();
            Assert.Equal("Controller Bindings", PadMenu.SelectedLine(window));
            pad.Start();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);

            // A new opening starts on Resume, and the submenu still remembers its row; B twice closes it.
            Chord(pad);
            Assert.Equal(0, PadMenu.Selected(window));
            Enter(window, pad, MainWindow.GameSettingsMenu);
            Assert.Equal("Controller Bindings", PadMenu.SelectedLine(window));
            pad.B();
            pad.B();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);
            Done(window);
        }, default);

        [Fact]
        public Task Left_and_right_on_save_and_load_change_the_slot_both_act_on_and_the_card_shows_what_it_holds() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: true);
            Chord(pad);
            pad.Down(2);
            Assert.Equal("Save State", PadMenu.Entries(window)[PadMenu.Selected(window)].Label!());
            WindowFitAuditTests.Settle(window);
            var card = window.GetControl<Control>("PadSlotCard");
            Assert.True(card.IsVisible);
            Assert.Equal("Slot 1\nEmpty", window.GetControl<TextBlock>("PadSlotCaption").Text);

            pad.Right();
            Assert.True(PadMenu.IsOpen(window));
            Assert.Equal(2, (int)Field(window, "_stateSlot")!);
            Assert.Equal("Slot 2 (Empty)", PadMenu.Entries(window)[2].Value!());
            Assert.Equal("Slot 2 (Empty)", PadMenu.Entries(window)[3].Value!());
            Assert.Equal("Slot 2\nEmpty", window.GetControl<TextBlock>("PadSlotCaption").Text);

            // A saves to slot 2 and closes the menu; slot 2 then holds a state and its picture, and Load State says so.
            pad.A();
            Assert.False(PadMenu.IsOpen(window));
            string state = SaveLibrary.StatePathFor(Running(window)!, 2, Path.Combine(_root, "States"));
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!File.Exists(SaveLibrary.PicturePathFor(state)) && clock.ElapsedMilliseconds < 20_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(File.Exists(state));
            Assert.False(File.Exists(SaveLibrary.StatePathFor(Running(window)!, 1, Path.Combine(_root, "States"))));
            Chord(pad);
            pad.Down(3);
            Assert.Equal("Load State      <  Slot 2  >", PadMenu.SelectedLine(window));
            WindowFitAuditTests.Settle(window);
            Assert.StartsWith("Slot 2\nSaved ", window.GetControl<TextBlock>("PadSlotCaption").Text);
            Assert.True(window.GetControl<Avalonia.Controls.Image>("PadSlotPicture").IsVisible);
            Assert.NotNull(window.GetControl<Avalonia.Controls.Image>("PadSlotPicture").Source);

            // Left back to slot 1 on the Load row moves both rows; the card goes with any other row.
            pad.Left();
            Assert.Equal(1, (int)Field(window, "_stateSlot")!);
            Assert.Equal("Slot 1 (Empty)", PadMenu.Entries(window)[2].Value!());
            pad.Down();
            WindowFitAuditTests.Settle(window);
            Assert.False(card.IsVisible);
            Done(window);
        }, default);

        // Each question opens on No, No and B change nothing, and Yes does what the row says.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Restart_quit_and_exit_ask_first_and_no_changes_nothing(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            object session = Field(window, "_session")!;

            Chord(pad);
            Enter(window, pad, "Restart Game");
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal([MainWindow.QuestionNo, MainWindow.QuestionYes], PadMenu.Lines(window));
            Assert.Equal(0, PadMenu.Selected(window));
            if (bigScreen)
            {
                Assert.Equal("Restart Game?", PadMenu.Big(window).Title);
                Assert.Contains("beginning", PadMenu.Big(window).Subtitle);
                Assert.Equal("No", PadMenu.Big(window).Hints![2].Label);
            }
            else Assert.Contains("beginning", window.GetControl<TextBlock>("PadMenuDetail").Text);
            pad.A();
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Equal("Restart Game...", PadMenu.SelectedLine(window));
            Assert.Same(session, Field(window, "_session"));
            pad.A();
            pad.B();
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Same(session, Field(window, "_session"));

            Enter(window, pad, "Quit Game");
            PadMenu.Answer(window, pad, yes: false);
            Assert.True(PadMenu.IsOpen(window));
            Assert.NotNull(Running(window));

            Enter(window, pad, MainWindow.EmuSenMenu);
            Enter(window, pad, "Exit EmuSen");
            Assert.Equal("Exit EmuSen?", PadMenu.Lines(window).Length == 2 ? (bigScreen ? PadMenu.Big(window).Title : "Exit EmuSen?") : null);
            PadMenu.Answer(window, pad, yes: false);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal("Exit EmuSen...", PadMenu.SelectedLine(window));
            Assert.True(window.IsVisible);
            pad.Start();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);

            // Yes: Restart gives a new session of the same game, running.
            Chord(pad);
            Enter(window, pad, "Restart Game");
            PadMenu.Answer(window, pad, yes: true);
            Assert.False(PadMenu.IsOpen(window));
            Assert.NotSame(session, Field(window, "_session"));
            Assert.NotNull(Running(window));
            Assert.False(window.IsPaused);

            // Quit closes the game to the library.
            Chord(pad);
            Enter(window, pad, "Quit Game");
            PadMenu.Answer(window, pad, yes: true);
            Assert.Null(Running(window));
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);

            // Exit closes the window.
            bool closed = false;
            window.Closed += (_, _) => closed = true;
            pad.Start();
            Enter(window, pad, MainWindow.EmuSenMenu);
            Enter(window, pad, "Exit EmuSen");
            PadMenu.Answer(window, pad, yes: true);
            Assert.True(closed);
        }, default);

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Back_to_library_leaves_the_game_paused_and_back_to_the_game_resumes_it(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            Chord(pad);
            PadMenu.Choose(window, pad, "Back to Library", exact: true);
            Assert.False(PadMenu.IsOpen(window));
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);
            Assert.True(window.IsPaused);
            Assert.NotNull(Running(window));
            for (int i = 0; i < 10; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            long frames = ((EmulatorSession)Field(window, "_session")!).TotalFrames;
            for (int i = 0; i < 20; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.Equal(frames, ((EmulatorSession)Field(window, "_session")!).TotalFrames);

            pad.Start();
            Assert.Equal("Back to Cobalt Harbor (Synthetic)", PadMenu.Lines(window)[0]);
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(window.IsPaused);
            Done(window);
        }, default);

        public static TheoryData<string, Type> InGameWindows() => new()
        {
            { "Cheats", typeof(ActiveCheatsWindow) }, { "Players & Controllers", typeof(PreferencesWindow) }, { "Controller Bindings", typeof(InputSettingsWindow) },
            { "Graphics", typeof(GraphicsSettingsWindow) }, { "Shaders", typeof(ShaderSettingsWindow) }, { "Preferences", typeof(PreferencesWindow) },
        };

        // Every row that moved into a submenu still opens its window, over the game, which stays paused under it.
        [Theory]
        [MemberData(nameof(InGameWindows))]
        public Task Each_moved_row_of_the_game_s_menu_opens_its_window(string row, Type window) => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing(bigScreen: true);
            Chord(pad);
            PadMenu.Choose(w, pad, row, exact: true);
            Assert.False(PadMenu.IsOpen(w));
            Assert.IsType(window, Sheets(w).Current);
            Assert.True(w.IsPaused);
            if (row == "Players & Controllers") Assert.Equal(PreferencesWindow.ControllersTab, PreferencesTab(w));
            Done(w);
        }, default);

        private static string? PreferencesTab(MainWindow w) =>
            (Sheets(w).SheetOf(Sheets(w).Current!)!.GetVisualDescendants().OfType<TabControl>().Single(t => t.Name == "PreferenceTabs").SelectedItem as TabItem)?.Header as string;

        public static TheoryData<string, Type> MainWindows() => new()
        {
            { "Players & Controllers", typeof(PreferencesWindow) }, { "Controller Bindings", typeof(InputSettingsWindow) }, { "Graphics", typeof(GraphicsSettingsWindow) },
            { "Shaders", typeof(ShaderSettingsWindow) }, { "Cheats", typeof(ActiveCheatsWindow) }, { "Preferences", typeof(PreferencesWindow) },
            { "Collections", typeof(CollectionSettingsWindow) }, { "Scrape Games...", typeof(PreferencesWindow) }, { "Theme Settings", typeof(ThemeSettingsWindow) },
        };

        [Theory]
        [MemberData(nameof(MainWindows))]
        public Task Each_moved_row_of_the_main_menu_opens_its_window(string row, Type window) => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            PadMenu.Choose(s.Window, s.Pad, row, exact: true);
            s.Settle();
            Assert.False(PadMenu.IsOpen(s.Window));
            Assert.IsType(window, Sheets(s.Window).Current);
            if (row == "Players & Controllers") Assert.Equal(PreferencesWindow.ControllersTab, PreferencesTab(s.Window));
            if (row == "Scrape Games...") Assert.Equal(PreferencesWindow.ScrapingTab, PreferencesTab(s.Window));
        }, default);

        // The desktop's keys steer the menu as the pad does, and the pointer chooses a row and goes back with its right button.
        [Fact]
        public Task Keys_and_the_pointer_drive_the_desktop_menu() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: false);
            Chord(pad);
            Press(window, Key.Up);
            Assert.Equal(MainWindow.EmuSenMenu, PadMenu.SelectedLine(window));
            Press(window, Key.Enter);
            Assert.Equal(2, PadMenu.Depth(window));
            Press(window, Key.Back);
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Equal(MainWindow.EmuSenMenu, PadMenu.SelectedLine(window));

            WindowFitAuditTests.Settle(window);
            Click(window, Array.IndexOf(PadMenu.Lines(window), MainWindow.GameSettingsMenu), MouseButton.Left);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal(GameSettings, PadMenu.Lines(window));
            Click(window, 0, MouseButton.Right);
            Assert.Equal(1, PadMenu.Depth(window));
            WindowFitAuditTests.Settle(window);
            Click(window, 0, MouseButton.Left);
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);
            Done(window);
        }, default);

        private static void Press(MainWindow w, Key key)
        {
            w.KeyPress(key, RawInputModifiers.None, PhysicalKey.None, null);
            w.KeyRelease(key, RawInputModifiers.None, PhysicalKey.None, null);
            Dispatcher.UIThread.RunJobs();
        }

        private static void Click(MainWindow w, int row, MouseButton button)
        {
            Control item = (Control)PadMenu.List(w).ContainerFromIndex(row)!;
            Point at = item.TranslatePoint(new Point(item.Bounds.Width / 2, item.Bounds.Height / 2), w)!.Value;
            w.MouseDown(at, button);
            w.MouseUp(at, button);
            Dispatcher.UIThread.RunJobs();
        }
    }
}
