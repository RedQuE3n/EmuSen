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

        internal static readonly string[] GameOptions = ["Cheats", MainWindow.ControllersMenu, MainWindow.DisplayMenu];
        internal static readonly string[] Leave = ["Back to Library", "Restart Game...", "Quit Game...", "Quit EmuSen..."];
        internal static readonly string[] Settings = ["Players & Controllers", "Controller Bindings", "Graphics", "Shaders", "Cheats", "Memory Cards", "Firmware", "Preferences"];

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
        public Task The_in_game_menu_is_the_game_s_actions_then_its_options_then_leaving(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            Chord(pad);
            string[] lines = PadMenu.Lines(window);
            Assert.Equal(7, lines.Length);
            Assert.Equal("Resume", lines[0]);
            Assert.StartsWith("Rewind", lines[1]);
            Assert.Equal("Save      <  Slot 1 (Empty)  >", lines[2]);
            Assert.Equal("Load      <  Slot 1 (Empty)  >", lines[3]);
            Assert.Equal("Speed      <  100%  >", lines[4]);
            Assert.Equal([MainWindow.GameOptionsMenu, MainWindow.LeaveMenu], lines[5..]);

            // One divider, before the options; Preferences, Shell and the EmuSen page are the library's alone.
            List<PadMenuEntry> rows = PadMenu.Entries(window);
            Assert.Equal([5], Enumerable.Range(0, rows.Count).Where(i => rows[i].StartsSection));
            Assert.All(new[] { 2, 3, 4 }, i => Assert.NotNull(rows[i].Adjust));
            Assert.Equal([MainWindow.SaveStateRow, MainWindow.LoadStateRow], rows.Where(r => r.ShowsSlot).Select(r => r.Label!()));
            Assert.Equal(["Rewind"], rows.Where(r => r.ShowsReel).Select(r => r.Label!()));
            Assert.DoesNotContain(PadMenu.AllLines(window), l => l is "Preferences" or MainWindow.ShellRow or MainWindow.ToolsMenu or MainWindow.PowerMenu or "Memory Cards");

            string[] options = SubmenuLines(window, MainWindow.GameOptionsMenu);
            Assert.Equal(GameOptions, options[..3]);
            Assert.StartsWith("Volume      <  ", options[3]);
            Assert.Equal(4, options.Length);
            Assert.Equal(Leave, SubmenuLines(window, MainWindow.LeaveMenu));
            Assert.Equal(["Restart Game?", "Quit Game?", "Quit EmuSen?"], rows.Single(r => r.Text() == MainWindow.LeaveMenu).Submenu!().Where(r => r.Question is not null).Select(r => r.Question!));

            // The card: the game's name as its title row, the pane beside it, the frame dimmed by the card's scrim and not blurred; the help bar from its actions.
            if (bigScreen)
            {
                Assert.True(PadMenu.Card(window).IsVisible);
                Assert.False(window.GetControl<Control>("MenuBackdrop").IsVisible);
                MenuPanel big = PadMenu.Big(window);
                Assert.Same(PadMenu.Card(window).CardPanel, big);
                Assert.Equal(MenuAnchor.Edge, big.Anchor);
                Assert.Equal("Cobalt Harbor (Synthetic)", big.Title);
                Assert.Null(big.Subtitle);
                Assert.Null(big.Footer);
                Assert.Equal(["Select", "Resume", "Screenshot", "Resume"], big.Hints!.Select(h => h.Label));
            }
            else Assert.Equal("Cobalt Harbor (Synthetic)", PadMenu.DeskTitle(window));
            Done(window);
        }, default);

        // Over the desktop's library with a game suspended: Continue, the game in hand, Search, Start Console, Tools, Settings and Power, on the card (EmuSen_BigPicture.md §49.1).
        [Fact]
        public Task The_home_card_is_continue_the_game_search_start_console_tools_settings_and_power() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: false);
            Call(window, "ToggleLibrary");
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.Start();
            Assert.Equal([MainWindow.ContinueRow, "Cobalt Harbor (Synthetic)", MainWindow.SearchRow, MainWindow.StartConsoleMenu, MainWindow.ToolsMenu, MainWindow.SettingsMenu, MainWindow.PowerMenu], PadMenu.Lines(window));
            Assert.Equal([4, 6], Enumerable.Range(0, 7).Where(i => PadMenu.Entries(window)[i].StartsSection));
            Assert.Equal([MainWindow.ScraperRow, MainWindow.PackagesMenu, MainWindow.LibraryManagerMenu, MainWindow.CoresMenu, MainWindow.CompressedDiscsMenu, MainWindow.ShellRow], SubmenuLines(window, MainWindow.ToolsMenu));
            Assert.Equal(Settings, SubmenuLines(window, MainWindow.SettingsMenu));
            Assert.Equal(["Full Screen", "Big Picture", MainWindow.QuitEmuSenRow], SubmenuLines(window, MainWindow.PowerMenu));
            Assert.True(PadMenu.Card(window).IsVisible);
            Assert.False(window.GetControl<Control>("PadMenuDesk").IsVisible);
            Assert.Equal(MainWindow.HomeTitle, PadMenu.Big(window).Title);
            Assert.Equal(["Choose", "Back", "Favourite", "Close"], PadMenu.Big(window).Hints!.Select(h => h.Label));
            Done(window);
        }, default);

        // Over the themed gamelist with no game running: no Continue, the gamelist's game and its search; the library's own tools under Tools, Exit Big Picture under Power.
        [Fact]
        public Task The_themed_home_card_folds_collections_and_theme_settings_into_tools() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession();
            ThemedLibraryPadTests.Enter(s, "snes");
            s.Pad.Start();
            Assert.Equal([s.Game!, MainWindow.SearchRow, MainWindow.StartConsoleMenu, MainWindow.ToolsMenu, MainWindow.SettingsMenu, MainWindow.PowerMenu], PadMenu.Lines(s.Window));
            Assert.Equal([MainWindow.ScraperRow, MainWindow.PackagesMenu, MainWindow.LibraryManagerMenu, MainWindow.CoresMenu, MainWindow.CompressedDiscsMenu, "Collections", MainWindow.ThemeSettingsRow, MainWindow.ShellRow], SubmenuLines(s.Window, MainWindow.ToolsMenu));
            Assert.Equal(["Exit Big Picture", MainWindow.QuitEmuSenRow], SubmenuLines(s.Window, MainWindow.PowerMenu));
            Assert.Equal(MainWindow.HomeTitle, PadMenu.Big(s.Window).Title);

            s.Pad.Up();
            Assert.Equal(MainWindow.PowerMenu, PadMenu.SelectedLine(s.Window));
            s.Pad.A();
            Assert.Equal(MainWindow.PowerMenu, PadMenu.Big(s.Window).Title);
            Assert.StartsWith("EmuSen 0.", PadMenu.Big(s.Window).Footer);
            Assert.Equal("Home", PadMenu.Big(s.Window).Subtitle);
            Assert.Equal(["Choose", "Back", "Close"], PadMenu.Big(s.Window).Hints!.Select(h => h.Label));
        }, default);

        // With a second pad, the first page offers Swap Players before the options, reached by pad; it trades players 1 and 2 and names both, and the page fits - see EmuSen_Input.md §8.13.
        [Theory]
        [InlineData(false, 1280, 800)]
        [InlineData(true, 1280, 800)]
        [InlineData(true, 1920, 1200)]
        public Task Swap_players_on_the_first_page_trades_players_1_and_2(bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen, width, height);
            pad.Pad.Name = "Legion Go S";
            PadDriver xbox = pad.Plug("Xbox Wireless Controller");
            pad.Tick();
            Assert.Equal(1, pad.Gamepad.Players.PlayerOf(pad.Gamepad.Pads[0]));
            Chord(pad);
            Assert.Equal(MainWindow.SwapPlayersRow, PadMenu.Lines(window)[5]);
            Assert.Equal(8, PadMenu.Lines(window).Length);
            pad.Down(5);
            WindowFitAuditTests.Settle(window);
            Control panel = bigScreen ? PadMenu.Big(window) : window.GetControl<Control>("PadMenuDesk");
            // The desktop menu keeps the desktop's text sizes, as the window audit allows it (EmuSen_Input.md §8.9).
            List<string> faults = bigScreen ? FitAudit.Check(panel) : FitAudit.Check(panel, smallest: 0);
            Assert.True(faults.Count == 0, string.Join("\n", faults));

            pad.A();
            Assert.False(PadMenu.IsOpen(window));
            Assert.Equal((2, 1), (pad.Gamepad.Players.PlayerOf(pad.Gamepad.Pads[0]), pad.Gamepad.Players.PlayerOf(pad.Gamepad.Pads[1])));
            Assert.Equal("Xbox Wireless Controller: player 1 · Legion Go S: player 2", window.GetControl<NoticeLayer>("PadNotice").Current);

            // The seats are the player's own now: a press on the built-in pad claims nothing for the rest of the game.
            pad.Gamepad.ClaimPressed();
            Assert.True(pad.Gamepad.PlayerOneClaimed);
            xbox.Unplug();
            pad.Tick();
            Done(window);
        }, default);

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task A_submenu_is_entered_with_A_left_with_B_to_the_row_it_was_entered_from_and_remembers_its_row(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            Chord(pad);
            Assert.True(window.IsPaused);
            Enter(window, pad, MainWindow.GameOptionsMenu);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal(GameOptions, PadMenu.Lines(window)[..3]);
            Assert.Equal(0, PadMenu.Selected(window));
            if (bigScreen)
            {
                Assert.Equal(MainWindow.GameOptionsMenu, PadMenu.Big(window).Title);
                Assert.Equal("Cobalt Harbor (Synthetic)", PadMenu.Big(window).Subtitle);
            }
            else Assert.Equal("Cobalt Harbor (Synthetic)  ›  Game Options", PadMenu.DeskTitle(window));

            pad.Down(2);
            pad.B();
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Equal(MainWindow.GameOptionsMenu, PadMenu.SelectedLine(window));
            Assert.True(window.IsPaused);

            // Entered again it opens on the row it was left on; Start closes the whole menu from inside it, and the game runs.
            pad.A();
            Assert.Equal(MainWindow.DisplayMenu, PadMenu.SelectedLine(window));
            pad.Start();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);

            // A new opening starts on Resume, and the submenu still remembers its row; B twice closes it.
            Chord(pad);
            Assert.Equal(0, PadMenu.Selected(window));
            Enter(window, pad, MainWindow.GameOptionsMenu);
            Assert.Equal(MainWindow.DisplayMenu, PadMenu.SelectedLine(window));
            pad.B();
            pad.B();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);
            Done(window);
        }, default);

        [Fact]
        public Task Left_and_right_on_save_and_load_change_the_slot_both_act_on_and_the_pane_shows_it_between_its_neighbours() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: true);
            Chord(pad);
            PauseCardPane pane = PadMenu.Pane(window);
            Assert.Equal("none", pane.Showing);
            pad.Down(2);
            Assert.Equal(MainWindow.SaveStateRow, PadMenu.Entries(window)[PadMenu.Selected(window)].Label!());
            WindowFitAuditTests.Settle(window);
            Assert.False(window.GetControl<Control>("PadSlotCard").IsVisible);
            Assert.Equal("slots", pane.Showing);
            Assert.Equal(Enumerable.Range(1, 3).Select(i => $"PaneSlot{i}"), PaneSlots(pane).Select(b => b.Name));
            Assert.Equal(1, Ringed(pane));
            Assert.Equal(6, PaneWords(pane).Count(t => t == "Empty"));
            Assert.Contains("A saves to Slot 1.", PaneWords(pane));

            pad.Right();
            Assert.True(PadMenu.IsOpen(window));
            Assert.Equal(2, (int)Field(window, "_stateSlot")!);
            Assert.Equal("Slot 2 (Empty)", PadMenu.Entries(window)[2].Value!());
            Assert.Equal("Slot 2 (Empty)", PadMenu.Entries(window)[3].Value!());
            WindowFitAuditTests.Settle(window);
            Assert.Equal(2, Ringed(pane));

            // A saves to slot 2 and closes the menu; slot 2 then holds a state and its picture, and the strip says so on Load.
            pad.A();
            Assert.False(PadMenu.IsOpen(window));
            string state = SaveLibrary.StatePathFor(Running(window)!, 2, Path.Combine(_root, "States"));
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!File.Exists(SaveLibrary.PicturePathFor(state)) && clock.ElapsedMilliseconds < 20_000) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.True(File.Exists(state));
            Assert.False(File.Exists(SaveLibrary.StatePathFor(Running(window)!, 1, Path.Combine(_root, "States"))));
            Chord(pad);
            pad.Down(3);
            Assert.Equal("Load      <  Slot 2  >", PadMenu.SelectedLine(window));
            WindowFitAuditTests.Settle(window);
            Assert.Equal("slots", pane.Showing);
            Assert.Equal(4, PaneWords(pane).Count(t => t == "Empty"));
            Assert.NotNull(PaneSlots(pane)[1].GetVisualDescendants().OfType<Avalonia.Controls.Image>().SingleOrDefault()?.Source);
            Assert.Empty(PaneSlots(pane)[0].GetVisualDescendants().OfType<Avalonia.Controls.Image>());
            Assert.Contains("A loads Slot 2.", PaneWords(pane));

            // Left back to slot 1 on the Load row moves both rows and the ring; the pane empties on any other row.
            pad.Left();
            Assert.Equal(1, (int)Field(window, "_stateSlot")!);
            Assert.Equal("Slot 1 (Empty)", PadMenu.Entries(window)[2].Value!());
            WindowFitAuditTests.Settle(window);
            Assert.Equal(1, Ringed(pane));
            pad.Down();
            WindowFitAuditTests.Settle(window);
            Assert.Equal("none", pane.Showing);
            Done(window);
        }, default);

        internal static List<Border> PaneSlots(PauseCardPane pane) => pane.GetVisualDescendants().OfType<Border>().Where(b => b.Name?.StartsWith("PaneSlot", StringComparison.Ordinal) == true && b.Name != "PaneSlots").ToList();

        // The slot whose frame is drawn thicker, as the current one is ringed.
        internal static int Ringed(PauseCardPane pane)
        {
            List<Border> slots = PaneSlots(pane);
            double most = slots.Max(b => b.BorderThickness.Left);
            Assert.Single(slots, b => b.BorderThickness.Left == most);
            return slots.FindIndex(b => b.BorderThickness.Left == most) + 1;
        }

        internal static IEnumerable<string> PaneWords(PauseCardPane pane) => pane.GetVisualDescendants().OfType<TextBlock>().Select(t => t.Text ?? "");

        // Each question opens on No, No and B change nothing, and Yes does what the row says.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Restart_quit_and_exit_ask_first_and_no_changes_nothing(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen);
            object session = Field(window, "_session")!;

            Chord(pad);
            Enter(window, pad, MainWindow.LeaveMenu);
            Enter(window, pad, "Restart Game");
            Assert.Equal(3, PadMenu.Depth(window));
            Assert.Equal([MainWindow.QuestionNo, MainWindow.QuestionYes], PadMenu.Lines(window));
            Assert.Equal(0, PadMenu.Selected(window));
            if (bigScreen)
            {
                Assert.Equal("Restart Game?", PadMenu.Big(window).Title);
                Assert.Contains("beginning", PadMenu.Big(window).Subtitle);
                Assert.Equal("Back", PadMenu.Big(window).Hints![1].Label);
            }
            else Assert.Contains("beginning", window.GetControl<TextBlock>("PadMenuDetail").Text);
            pad.A();
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal("Restart Game...", PadMenu.SelectedLine(window));
            Assert.Same(session, Field(window, "_session"));
            pad.A();
            pad.B();
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Same(session, Field(window, "_session"));

            Enter(window, pad, "Quit Game");
            PadMenu.Answer(window, pad, yes: false);
            Assert.True(PadMenu.IsOpen(window));
            Assert.NotNull(Running(window));

            Enter(window, pad, "Quit EmuSen");
            Assert.Equal("Quit EmuSen?", PadMenu.Lines(window).Length == 2 ? (bigScreen ? PadMenu.Big(window).Title : "Quit EmuSen?") : null);
            PadMenu.Answer(window, pad, yes: false);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal("Quit EmuSen...", PadMenu.SelectedLine(window));
            Assert.True(window.IsVisible);
            pad.Start();
            Assert.False(PadMenu.IsOpen(window));
            Assert.False(window.IsPaused);

            // Yes: Restart presses the console's reset, VenusRT claiming one, so the session stays and runs on (decided 2026-10-18).
            Chord(pad);
            Enter(window, pad, MainWindow.LeaveMenu);
            Enter(window, pad, "Restart Game");
            PadMenu.Answer(window, pad, yes: true);
            Assert.False(PadMenu.IsOpen(window));
            Assert.Same(session, Field(window, "_session"));
            Assert.NotNull(Running(window));
            Assert.False(window.IsPaused);

            // Quit closes the game to the library.
            Chord(pad);
            Enter(window, pad, MainWindow.LeaveMenu);
            Enter(window, pad, "Quit Game");
            PadMenu.Answer(window, pad, yes: true);
            Assert.Null(Running(window));
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);

            // Exit closes the window.
            bool closed = false;
            window.Closed += (_, _) => closed = true;
            pad.Start();
            Enter(window, pad, MainWindow.PowerMenu);
            Enter(window, pad, "Quit EmuSen");
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
            long frames = MistressGame.TotalFrames(window);
            for (int i = 0; i < 20; i++) { Dispatcher.UIThread.RunJobs(); Thread.Sleep(5); }
            Assert.Equal(frames, MistressGame.TotalFrames(window));

            pad.Start();
            Assert.Equal(MainWindow.ContinueRow, PadMenu.Lines(window)[0]);
            WindowFitAuditTests.Settle(window);
            Assert.True(PadMenu.Card(window).IsVisible);
            Assert.Equal("frame", PadMenu.Pane(window).Showing);
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(window.IsPaused);
            Done(window);
        }, default);

        public static TheoryData<string, Type> InGameWindows() => new()
        {
            { "Cheats", typeof(ActiveCheatsWindow) }, { "Players & Controllers", typeof(PreferencesWindow) }, { "Controller Bindings", typeof(InputSettingsWindow) },
            { "Graphics", typeof(GraphicsSettingsWindow) }, { "Shaders", typeof(ShaderSettingsWindow) },
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

        // §10y: a game the card paused runs no frame from the card into a sheet and back to the card; it resumes only when the card closes.
        [Fact]
        public Task A_game_the_card_paused_runs_no_frame_from_the_card_into_a_sheet_and_back() => Session.Dispatch(() =>
        {
            (MainWindow w, PadDriver pad) = Playing(bigScreen: true);
            var said = new List<string?>();
            w.GetControl<TextBlock>("StatusText").PropertyChanged += (_, e) => { if (e.Property == TextBlock.TextProperty) said.Add(e.NewValue as string); };
            Chord(pad);
            PadMenu.Choose(w, pad, "Graphics", exact: true);
            Assert.IsType<GraphicsSettingsWindow>(Sheets(w).Current);
            Assert.True(w.IsPaused);
            ((Window)Sheets(w).Current!).Close();
            Dispatcher.UIThread.RunJobs();
            Assert.True(PadMenu.IsOpen(w), "closing the sheet comes back to the card");
            Assert.True(w.IsPaused);
            Assert.DoesNotContain(said, t => t?.StartsWith("Running", StringComparison.Ordinal) == true);
            pad.B();
            Assert.False(PadMenu.IsOpen(w));
            Assert.False(w.IsPaused);
            Assert.StartsWith("Running", said[^1]);
            Done(w);
        }, default);

        private static string? PreferencesTab(MainWindow w) =>
            (Sheets(w).SheetOf(Sheets(w).Current!)!.GetVisualDescendants().OfType<TabControl>().Single(t => t.Name == "PreferenceTabs").SelectedItem as TabItem)?.Header as string;

        public static TheoryData<string, Type> MainWindows() => new()
        {
            { "Players & Controllers", typeof(PreferencesWindow) }, { "Controller Bindings", typeof(InputSettingsWindow) }, { "Graphics", typeof(GraphicsSettingsWindow) },
            { "Shaders", typeof(ShaderSettingsWindow) }, { "Cheats", typeof(ActiveCheatsWindow) }, { "Preferences", typeof(PreferencesWindow) },
            { "Collections", typeof(CollectionSettingsWindow) }, { MainWindow.ScraperRow, typeof(ScraperWindow) }, { "Theme Settings", typeof(ThemeSettingsWindow) },
            { "Memory Cards", typeof(MemoryCardsWindow) }, { "Compressed Discs", typeof(CompressedDiscsWindow) }, { "Packages", typeof(PackagesWindow) }, { "Library Manager", typeof(LibraryWindow) }, { "Cores", typeof(CoresWindow) }, { "Shell", typeof(HotaruWindow) },
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
        }, default);

        // The desktop's keys steer the menu as the pad does, and the pointer chooses a row and goes back with its right button.
        [Fact]
        public Task Keys_and_the_pointer_drive_the_desktop_menu() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Playing(bigScreen: false);
            Chord(pad);
            Press(window, Key.Up);
            Assert.Equal(MainWindow.LeaveMenu, PadMenu.SelectedLine(window));
            Press(window, Key.Enter);
            Assert.Equal(2, PadMenu.Depth(window));
            Press(window, Key.Back);
            Assert.Equal(1, PadMenu.Depth(window));
            Assert.Equal(MainWindow.LeaveMenu, PadMenu.SelectedLine(window));

            WindowFitAuditTests.Settle(window);
            Click(window, Array.IndexOf(PadMenu.Lines(window), MainWindow.GameOptionsMenu), MouseButton.Left);
            Assert.Equal(2, PadMenu.Depth(window));
            Assert.Equal(GameOptions, PadMenu.Lines(window)[..3]);
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
