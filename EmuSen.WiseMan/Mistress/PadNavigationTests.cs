using EmuSen.Galaxia.Library;
using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.LunaP;

namespace EmuSen.WiseMan.Mistress
{
    // The interface steered from a pad: presses and repeats, the library, the menu, and another window through its keys - see EmuSen_Settings_Reference.md §4.29.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadNavigationTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadNavigationTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadNavigationTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadNavigationTests()
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

        private static TimeSpan Ms(int ms) => TimeSpan.FromMilliseconds(ms);

        [Fact]
        public void A_held_direction_presses_once_then_repeats_after_the_delay()
        {
            var navigator = new PadNavigator();
            Func<UiButton, bool> down = b => b == UiButton.Down;

            Assert.Equal(new[] { UiButton.Down }, navigator.Feed(down, Ms(0)));
            Assert.Empty(navigator.Feed(down, Ms(399)));
            Assert.Equal(new[] { UiButton.Down }, navigator.Feed(down, Ms(400)));
            Assert.Empty(navigator.Feed(down, Ms(479)));
            Assert.Equal(new[] { UiButton.Down }, navigator.Feed(down, Ms(480)));
        }

        [Fact]
        public void Accept_never_repeats_however_long_it_is_held()
        {
            var navigator = new PadNavigator();
            Func<UiButton, bool> accept = b => b == UiButton.Accept;

            Assert.Equal(new[] { UiButton.Accept }, navigator.Feed(accept, Ms(0)));
            Assert.Empty(navigator.Feed(accept, Ms(5000)));
            Assert.Empty(navigator.Feed(_ => false, Ms(5016)));
            Assert.Equal(new[] { UiButton.Accept }, navigator.Feed(accept, Ms(5032)));
        }

        [Fact]
        public void The_chord_is_the_guide_button_or_both_middle_buttons_and_fires_once()
        {
            var navigator = new PadNavigator();

            Assert.False(navigator.MenuChord(b => b == UiButton.Menu));
            Assert.True(navigator.MenuChord(b => b is UiButton.Menu or UiButton.Options));
            Assert.False(navigator.MenuChord(b => b is UiButton.Menu or UiButton.Options));
            Assert.False(navigator.MenuChord(_ => false));
            Assert.True(navigator.MenuChord(b => b == UiButton.Guide));
        }

        [Fact]
        public void A_button_already_down_when_forgotten_is_not_a_press_until_let_go()
        {
            var navigator = new PadNavigator();
            Func<UiButton, bool> back = b => b == UiButton.Back;

            navigator.Forget(back);
            Assert.Empty(navigator.Feed(back, Ms(0)));
            Assert.Empty(navigator.Feed(_ => false, Ms(16)));
            Assert.Equal(new[] { UiButton.Back }, navigator.Feed(back, Ms(32)));
        }

        private MainWindow LibraryOf(int games)
        {
            for (int i = 0; i < games; i++)
                File.WriteAllBytes(Path.Combine(_romDir, $"Game {i:00}.sfc"), SyntheticRom.BuildBlank());
            // The list's grammar; the covers' is LibraryScreenTests' - see EmuSen_Settings_Reference.md §4.33.
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList }.Save();

            var window = new MainWindow();
            window.Show();
            return window;
        }

        private static ListBox Library(MainWindow w) => w.GetControl<ListBox>("LibraryList");
        private static ListBox Menu(MainWindow w) => w.GetControl<ListBox>("PadMenuList");
        private static Control MenuPanel(MainWindow w) => w.GetControl<Control>("PadMenuPanel");

        private static void Pad(MainWindow window, params UiButton[] buttons)
        {
            MethodInfo command = typeof(MainWindow).GetMethod("OnPadCommand", BindingFlags.Instance | BindingFlags.NonPublic)!;
            foreach (UiButton button in buttons) command.Invoke(window, new object[] { button });
        }

        private static string[] MenuLines(MainWindow w) => Menu(w).ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        [Fact]
        public Task The_library_moves_by_one_by_ten_and_to_either_end() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(25);
            Library(window).SelectedIndex = 0;

            Pad(window, UiButton.Down, UiButton.Down);
            Assert.Equal(2, Library(window).SelectedIndex);

            Pad(window, UiButton.PageDown);
            Assert.Equal(12, Library(window).SelectedIndex);

            Pad(window, UiButton.Last);
            Assert.Equal(24, Library(window).SelectedIndex);

            Pad(window, UiButton.Down);
            Assert.Equal(24, Library(window).SelectedIndex);

            Pad(window, UiButton.PageUp, UiButton.Up);
            Assert.Equal(13, Library(window).SelectedIndex);

            Pad(window, UiButton.First);
            Assert.Equal(0, Library(window).SelectedIndex);
        }, default);

        [Fact]
        public Task Left_and_right_step_the_console_filter_round_and_the_choice_is_kept() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(2);
            var choices = EmuSen.Cores.CoreCatalog.FilterChoices;
            var filter = window.GetControl<FilterBar>("LibraryFilter");
            int at = choices.ToList().IndexOf((string)filter.Facet!);

            // The first step lands on the legacy default, which is how EmuSen_Multicore.md §10.3a was found.
            Pad(window, UiButton.Right);
            Assert.Equal(choices[(at + 1) % choices.Count], filter.Facet);
            Assert.Equal(choices[(at + 1) % choices.Count], AppSettings.Load().SelectedCore);

            Pad(window, UiButton.Left, UiButton.Left);
            Assert.Equal(choices[(at - 1 + choices.Count) % choices.Count], filter.Facet);
        }, default);

        [Fact]
        public Task Accept_starts_the_selected_game_and_the_menu_over_it_pauses_and_resumes() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(3);
            Library(window).SelectedIndex = 1;

            Pad(window, UiButton.Accept);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            Assert.False(IsPaused(window));

            Invoke(window, "OpenPadMenu");
            Assert.True(MenuPanel(window).IsVisible);
            Assert.True(IsPaused(window));
            Assert.Equal("Resume", MenuLines(window)[0]);

            Pad(window, UiButton.Back);
            Assert.False(MenuPanel(window).IsVisible);
            Assert.False(IsPaused(window));

            Invoke(window, "StopEmulationThread");
        }, default);

        [Fact]
        public Task A_game_the_player_had_paused_stays_paused_when_the_menu_closes() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(1);
            Library(window).SelectedIndex = 0;
            Pad(window, UiButton.Accept);
            window.PauseEmulation();

            Invoke(window, "OpenPadMenu");
            Pad(window, UiButton.Back);
            Assert.True(IsPaused(window));

            Invoke(window, "StopEmulationThread");
        }, default);

        [Fact]
        public Task The_menu_wraps_and_left_and_right_change_the_state_slot_without_closing_it() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(1);
            Library(window).SelectedIndex = 0;
            Pad(window, UiButton.Accept);
            Invoke(window, "OpenPadMenu");

            Pad(window, UiButton.Up);
            Assert.Equal(MenuLines(window).Length - 1, Menu(window).SelectedIndex);
            Pad(window, UiButton.Down);
            Assert.Equal(0, Menu(window).SelectedIndex);

            int slotLine = Array.FindIndex(MenuLines(window), line => line.StartsWith("State Slot"));
            Menu(window).SelectedIndex = slotLine;
            Pad(window, UiButton.Right);

            Assert.True(MenuPanel(window).IsVisible);
            Assert.Equal(slotLine, Menu(window).SelectedIndex);
            Assert.Contains("<  2  >", MenuLines(window)[slotLine]);
            Assert.Contains("slot 2", MenuLines(window)[1]);

            Pad(window, UiButton.Left, UiButton.Left);
            Assert.Contains("<  8  >", MenuLines(window)[slotLine]);

            Invoke(window, "StopEmulationThread");
        }, default);

        [Fact]
        public Task Start_in_the_library_opens_a_menu_with_no_game_entries_and_the_game_hears_nothing_while_it_is_up() => Session.Dispatch(() =>
        {
            MainWindow window = LibraryOf(1);

            Pad(window, UiButton.Menu);
            Assert.True(MenuPanel(window).IsVisible);
            Assert.DoesNotContain("Resume", MenuLines(window));
            Assert.Contains("Graphics Settings", MenuLines(window));
            Assert.True(BelongsToTheInterface(window));

            Pad(window, UiButton.Menu);
            Assert.False(MenuPanel(window).IsVisible);
        }, default);

        [Fact]
        public Task Another_window_is_driven_through_its_keys() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), null, "N64");
            window.Show();
            window.CaptureRenderedFrame();

            var tabs = window.GetVisualDescendants().OfType<Tabs>().First();
            int tab = tabs.SelectedIndex;
            PadWindowRouter.Send(window, UiButton.PageDown);
            Assert.Equal((tab + 1) % tabs.ItemCount, tabs.SelectedIndex);
            PadWindowRouter.Send(window, UiButton.PageUp);
            Assert.Equal(tab, tabs.SelectedIndex);

            // Down walks the focus; somewhere along it is a dropdown, and right moves it by one.
            Dropdown? dropdown = null;
            for (int i = 0; i < 40 && dropdown is null; i++)
            {
                PadWindowRouter.Send(window, UiButton.Down);
                dropdown = window.FocusManager?.GetFocusedElement() as Dropdown;
            }

            Assert.NotNull(dropdown);
            int before = dropdown!.SelectedIndex;
            PadWindowRouter.Send(window, UiButton.Right);
            Assert.Equal(before + 1, dropdown.SelectedIndex);

            PadWindowRouter.Send(window, UiButton.Accept);
            Assert.True(dropdown.IsDropDownOpen);
            PadWindowRouter.Send(window, UiButton.Back);
            Assert.False(dropdown.IsDropDownOpen);
            Assert.True(window.IsVisible);

            PadWindowRouter.Send(window, UiButton.Back);
            Assert.False(window.IsVisible);
        }, default);

        [Fact]
        public Task Accept_toggles_a_focused_switch() => Session.Dispatch(() =>
        {
            var window = new GraphicsSettingsWindow(new GraphicsConfig(), null, "N64");
            window.Show();
            window.CaptureRenderedFrame();

            LunaSwitch toggle = window.GetVisualDescendants().OfType<LunaSwitch>().First(s => s.IsEffectivelyVisible);
            toggle.Focus();
            bool before = toggle.IsChecked == true;

            PadWindowRouter.Send(window, UiButton.Accept);
            Assert.Equal(!before, toggle.IsChecked == true);
        }, default);

        // Steam's keyboard with its three questions answered by the test, and what it was asked to open.
        private sealed class FakeSteam : IDisposable
        {
            private readonly Action<string> _launcher = SteamKeyboard.Launcher;
            private readonly Func<string, string?> _environment = SteamKeyboard.Environment;
            private readonly Func<bool> _running = SteamKeyboard.ClientRunning;

            public System.Collections.Generic.List<string> Opened { get; } = new();

            public FakeSteam(bool running = false, string? deck = null, string? gameId = null)
            {
                SteamKeyboard.Launcher = Opened.Add;
                SteamKeyboard.ClientRunning = () => running;
                SteamKeyboard.Environment = name => name == "SteamDeck" ? deck : name == "SteamGameId" ? gameId : null;
            }

            public void Dispose()
            {
                SteamKeyboard.Launcher = _launcher;
                SteamKeyboard.Environment = _environment;
                SteamKeyboard.ClientRunning = _running;
            }
        }

        [Fact]
        public void The_keyboard_is_asked_of_a_steam_that_is_there_and_never_of_one_that_is_not()
        {
            using (var steam = new FakeSteam())
            {
                Assert.False(SteamKeyboard.Show());
                Assert.Empty(steam.Opened);
            }

            using (var steam = new FakeSteam(running: true))
            {
                Assert.True(SteamKeyboard.Show());
                Assert.Equal(new[] { "steam://open/keyboard" }, steam.Opened);
            }

            using (var steam = new FakeSteam(deck: "1")) Assert.True(SteamKeyboard.Show());
            using (var steam = new FakeSteam(gameId: "12345")) Assert.True(SteamKeyboard.Show());
        }

        [Fact]
        public void A_launcher_that_throws_is_a_keyboard_that_did_not_open_and_nothing_more()
        {
            using var steam = new FakeSteam(running: true);
            SteamKeyboard.Launcher = _ => throw new System.ComponentModel.Win32Exception("no steam");

            Assert.False(SteamKeyboard.Show());
        }

        [Fact]
        public Task Search_focuses_the_box_and_asks_for_the_keyboard_the_list_still_moves_and_back_returns_to_it() => Session.Dispatch(() =>
        {
            using var steam = new FakeSteam(running: true);
            MainWindow window = LibraryOf(5);
            Library(window).SelectedIndex = 0;

            Pad(window, UiButton.Search);
            Assert.IsType<TextBox>(window.FocusManager!.GetFocusedElement());
            Assert.Single(steam.Opened);

            Pad(window, UiButton.Down);
            Assert.Equal(1, Library(window).SelectedIndex);
            Assert.IsType<TextBox>(window.FocusManager!.GetFocusedElement());

            Pad(window, UiButton.Back);
            Assert.IsNotType<TextBox>(window.FocusManager!.GetFocusedElement());
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);
        }, default);

        [Fact]
        public Task Accept_on_a_text_box_in_another_window_asks_for_the_keyboard() => Session.Dispatch(() =>
        {
            using var steam = new FakeSteam(running: true);
            var window = new PreferencesWindow(new AppSettings());
            window.Show();
            window.CaptureRenderedFrame();

            TextBox box = window.GetVisualDescendants().OfType<TextBox>().First();
            box.Focus();
            PadWindowRouter.Send(window, UiButton.Accept);

            Assert.Single(steam.Opened);
            Assert.True(window.IsVisible);
        }, default);

        private static bool IsPaused(MainWindow window) =>
            (bool)typeof(MainWindow).GetProperty("IsPaused", BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public)!.GetValue(window)!;

        private static bool BelongsToTheInterface(MainWindow window) =>
            (bool)typeof(MainWindow).GetMethod("PadBelongsToTheInterface", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null)!;

        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
    }
}
