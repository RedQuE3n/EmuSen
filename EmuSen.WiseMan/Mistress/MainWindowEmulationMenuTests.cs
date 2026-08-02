using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Pause/Reset/Close Game - see EmuSen_Settings_Reference.md §4.12.
    public class MainWindowEmulationMenuTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowEmulationMenuTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;
        private readonly string _stateDir;

        public MainWindowEmulationMenuTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenMainWindowEmulationTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            _stateDir = Path.Combine(_root, "States");
            // Or every load in here writes into the real user log/state trees.
            new AppSettings { RomDirectory = _romDir, LogDirectory = Path.Combine(_root, "Logs"), StateDirectory = _stateDir }.Save();
            File.WriteAllBytes(Path.Combine(_romDir, "Playable.smc"), SyntheticRom.BuildBlank());
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static MenuItem Item(MainWindow w, string name) => w.GetControl<MenuItem>(name);
        private static TextBlock Status(MainWindow w) => w.GetControl<TextBlock>("StatusText");
        private static Control LibraryView(MainWindow w) => w.GetControl<DockPanel>("LibraryView");
        private static Control GameFrame(MainWindow w) => w.GetControl<Control>("GameFrame");

        // The real XAML-wired handler, not the private method behind it.
        private static void OpenEmulationMenu(MainWindow w) =>
            Item(w, "EmulationMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));

        private static void Click(MainWindow w, string name) =>
            Item(w, name).RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));

        private static MainWindow StartGame()
        {
            var window = new MainWindow();
            window.Show();
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            return window;
        }

        [Fact]
        public Task Every_emulation_item_is_disabled_until_a_game_is_running() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            OpenEmulationMenu(window);

            foreach (string name in new[] { "PauseMenuItem", "ResetMenuItem", "CloseGameMenuItem", "SaveStateMenuItem", "LoadStateMenuItem" })
            {
                Assert.False(Item(window, name).IsEnabled, $"{name} should be disabled with no ROM loaded.");
            }
        }, default);

        [Fact]
        public Task Starting_a_game_enables_them() => Session.Dispatch(() =>
        {
            var window = StartGame();

            OpenEmulationMenu(window);

            foreach (string name in new[] { "PauseMenuItem", "ResetMenuItem", "CloseGameMenuItem", "SaveStateMenuItem", "LoadStateMenuItem" })
            {
                Assert.True(Item(window, name).IsEnabled, $"{name} should be enabled while a game runs.");
            }

            window.Close();
        }, default);

        [Fact]
        public Task Pause_halts_the_emulation_thread_and_clicking_again_resumes_it() => Session.Dispatch(() =>
        {
            var window = StartGame();
            Assert.False(window.IsPaused);

            Click(window, "PauseMenuItem");
            Assert.True(window.IsPaused);
            Assert.Equal("Paused", Status(window).Text);

            Click(window, "PauseMenuItem");
            Assert.False(window.IsPaused);
            Assert.Contains("Playable.smc", Status(window).Text!);

            window.Close();
        }, default);

        // The check mark is only ever synced on open - see §4.12.
        [Fact]
        public Task The_pause_item_shows_checked_while_paused() => Session.Dispatch(() =>
        {
            var window = StartGame();

            OpenEmulationMenu(window);
            Assert.False(Item(window, "PauseMenuItem").IsChecked);

            window.PauseEmulation();
            OpenEmulationMenu(window);
            Assert.True(Item(window, "PauseMenuItem").IsChecked);

            window.ResumeEmulation();
            OpenEmulationMenu(window);
            Assert.False(Item(window, "PauseMenuItem").IsChecked);

            window.Close();
        }, default);

        [Fact]
        public Task Reset_rebuilds_the_machine_and_stays_on_the_game_screen() => Session.Dispatch(() =>
        {
            var window = StartGame();
            object? before = Field(window, "_session");
            Assert.NotNull(before);

            Click(window, "ResetMenuItem");

            Assert.NotSame(before, Field(window, "_session"));
            Assert.Equal("Reset: Playable.smc", Status(window).Text);
            Assert.True(GameFrame(window).IsVisible);
            Assert.False(LibraryView(window).IsVisible);

            window.Close();
        }, default);

        [Fact]
        public Task Reset_comes_back_running_even_if_it_was_paused() => Session.Dispatch(() =>
        {
            var window = StartGame();
            window.PauseEmulation();
            Assert.True(window.IsPaused);

            Click(window, "ResetMenuItem");

            Assert.False(window.IsPaused);

            window.Close();
        }, default);

        [Fact]
        public Task Close_game_drops_the_session_and_returns_to_the_library() => Session.Dispatch(() =>
        {
            var window = StartGame();
            Assert.True(GameFrame(window).IsVisible);

            Click(window, "CloseGameMenuItem");

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(GameFrame(window).IsVisible);
            Assert.Equal("No ROM loaded", Status(window).Text);
            Assert.Null(Field(window, "_session"));
            // Or an open console/dashboard keeps inspecting the core we dropped.
            Assert.Null(Field(window, "_debugTarget"));

            OpenEmulationMenu(window);
            Assert.False(Item(window, "CloseGameMenuItem").IsEnabled);

            window.Close();
        }, default);

        [Fact]
        public Task A_game_can_be_started_again_after_being_closed() => Session.Dispatch(() =>
        {
            var window = StartGame();
            Click(window, "CloseGameMenuItem");

            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");

            Assert.True(GameFrame(window).IsVisible);
            Assert.NotNull(Field(window, "_session"));
            Assert.Contains("Playable.smc", Status(window).Text!);

            window.Close();
        }, default);

        // --- Speed ---

        [Fact]
        public Task The_speed_menu_sets_the_base_speed_the_emulation_loop_reads() => Session.Dispatch(() =>
        {
            var window = StartGame();
            Assert.Equal(100, Field(window, "_baseSpeedPercent"));

            Click(window, "SpeedFastMenuItem");
            Assert.Equal(300, Field(window, "_baseSpeedPercent"));

            Click(window, "SpeedSlowMenuItem");
            Assert.Equal(25, Field(window, "_baseSpeedPercent"));

            Click(window, "SpeedUnthrottledMenuItem");
            Assert.Equal(0, Field(window, "_baseSpeedPercent"));

            Click(window, "SpeedNormalMenuItem");
            Assert.Equal(100, Field(window, "_baseSpeedPercent"));

            window.Close();
        }, default);

        [Fact]
        public Task The_speed_menu_checks_whichever_speed_is_current() => Session.Dispatch(() =>
        {
            var window = StartGame();

            Click(window, "SpeedSlowMenuItem");
            OpenEmulationMenu(window);

            Assert.True(Item(window, "SpeedSlowMenuItem").IsChecked);
            Assert.False(Item(window, "SpeedNormalMenuItem").IsChecked);
            Assert.False(Item(window, "SpeedFastMenuItem").IsChecked);
            Assert.False(Item(window, "SpeedUnthrottledMenuItem").IsChecked);

            window.Close();
        }, default);

        // --- Save slots ---

        [Fact]
        public Task Slot_one_writes_the_plain_state_file_every_other_frontend_uses() => Session.Dispatch(() =>
        {
            var window = StartGame();
            window.PauseEmulation(); // SaveState reads core state the emulation thread is writing

            Click(window, "SaveStateMenuItem");

            Assert.True(File.Exists(Path.Combine(_stateDir, "Playable.state")));
            Assert.Contains("slot 1", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task A_second_slot_writes_its_own_file_and_leaves_the_first_alone() => Session.Dispatch(() =>
        {
            var window = StartGame();
            window.PauseEmulation();
            Click(window, "SaveStateMenuItem");

            SelectSlot(window, 3);
            Click(window, "SaveStateMenuItem");

            Assert.True(File.Exists(Path.Combine(_stateDir, "Playable.state")));
            Assert.True(File.Exists(Path.Combine(_stateDir, "Playable.slot3.state")));
            Assert.Contains("slot 3", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task Loading_an_untouched_slot_says_it_is_empty() => Session.Dispatch(() =>
        {
            var window = StartGame();
            SelectSlot(window, 5);

            Click(window, "LoadStateMenuItem");

            Assert.Equal("Load State: slot 5 is empty", Status(window).Text);

            window.Close();
        }, default);

        [Fact]
        public Task The_slot_menu_reports_which_slots_hold_a_state() => Session.Dispatch(() =>
        {
            var window = StartGame();
            window.PauseEmulation();
            SelectSlot(window, 2);
            Click(window, "SaveStateMenuItem");

            var slots = SlotItems(window);
            Assert.Equal(8, slots.Count);
            Assert.Contains("empty", (string)slots[0].Header!);
            Assert.DoesNotContain("empty", (string)slots[1].Header!);
            Assert.True(slots[1].IsChecked);

            window.Close();
        }, default);

        // --- View and Settings ---

        [Fact]
        public Task Fullscreen_toggles_the_window_state_both_ways() => Session.Dispatch(() =>
        {
            var window = StartGame();

            Click(window, "FullscreenMenuItem");
            Assert.Equal(WindowState.FullScreen, window.WindowState);
            Item(window, "ViewMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            Assert.True(Item(window, "FullscreenMenuItem").IsChecked);

            Click(window, "FullscreenMenuItem");
            Assert.Equal(WindowState.Normal, window.WindowState);

            window.Close();
        }, default);

        [Fact]
        public Task The_hardware_dashboard_needs_a_debug_target() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            Item(window, "SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            Assert.False(Item(window, "HardwareDashboardMenuItem").IsEnabled);

            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Item(window, "SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            Assert.True(Item(window, "HardwareDashboardMenuItem").IsEnabled);

            window.Close();
        }, default);

        private static System.Collections.Generic.List<MenuItem> SlotItems(MainWindow w)
        {
            var slots = Item(w, "SaveSlotMenuItem");
            slots.RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            return ((System.Collections.Generic.IEnumerable<MenuItem>)slots.ItemsSource!).ToList();
        }

        private static void SelectSlot(MainWindow w, int slot) =>
            SlotItems(w)[slot - 1].Command!.Execute(null);

        // Private by design - the emulation menu is internal to MainWindow, not API.
        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static object? Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);
    }
}
