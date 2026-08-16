using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Commands;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Pause/Reset/Close Game - see EmuSen_Settings_Reference.md §4.12.
    [Collection(TestCollections.ProcessGlobals)]
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

        // The action the menu item follows, under the MenuItem's old name - see §4.12.
        private static LunaAction Item(MainWindow w, string name) => Actions(w)[name];

        private static TextBlock Status(MainWindow w) => w.GetControl<TextBlock>("StatusText");
        private static Control LibraryView(MainWindow w) => w.GetControl<DockPanel>("LibraryView");
        private static Control GameFrame(MainWindow w) => w.GetControl<Control>("GameFrame");

        // Invoke is the whole click: a checkable action flips itself first.
        private static void Click(MainWindow w, string name) => Item(w, name).Invoke();

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

            foreach (string name in new[] { "PauseMenuItem", "ResetMenuItem", "CloseGameMenuItem", "SaveStateMenuItem", "LoadStateMenuItem" })
            {
                Assert.False(Item(window, name).IsEnabled, $"{name} should be disabled with no ROM loaded.");
            }
        }, default);

        [Fact]
        public Task Starting_a_game_enables_them() => Session.Dispatch(() =>
        {
            var window = StartGame();

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

        // No menu is opened here: the tick follows the state itself - see §4.12.
        [Fact]
        public Task The_pause_item_shows_checked_while_paused() => Session.Dispatch(() =>
        {
            var window = StartGame();

            Assert.False(Item(window, "PauseMenuItem").IsChecked);

            window.PauseEmulation();
            Assert.True(Item(window, "PauseMenuItem").IsChecked);

            window.ResumeEmulation();
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
            Assert.Contains("empty", slots[0].Text);
            Assert.DoesNotContain("empty", slots[1].Text);
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
            Assert.True(Item(window, "FullscreenMenuItem").IsChecked);

            Click(window, "FullscreenMenuItem");
            Assert.Equal(WindowState.Normal, window.WindowState);

            window.Close();
        }, default);

        // Every other test here would pass against a menu bar that drew nothing - see §4.12.
        [Fact]
        public Task The_menu_bar_really_builds_its_items() => Session.Dispatch(() =>
        {
            var window = StartGame();

            var bar = window.GetControl<EmuSen.LunaP.Controls.MenuBar>("MenuStrip");
            Assert.Equal(
                new[] { "_File", "_Emulation", "_View", "_Settings" },
                bar.Menus.Select(m => m.Title).ToArray());

            // Submenu items are realised on open, so open it the way a pointer would.
            MenuItem emulation = bar.GetVisualDescendants().OfType<MenuItem>()
                .Single(i => (i.Header as string) == "_Emulation");
            emulation.Open();

            // Disabling the action disables the item somebody would click.
            MenuItem pause = emulation.GetLogicalDescendants().OfType<MenuItem>()
                .Single(i => (i.Header as string) == "_Pause");
            Assert.True(pause.IsEnabled);
            Click(window, "CloseGameMenuItem");
            Assert.False(pause.IsEnabled);

            window.Close();
        }, default);

        // The menu advertises the hotkeys, driven from HotkeyBindingMap - see §4.19.
        [Fact]
        public Task The_menu_shows_the_key_each_command_is_bound_to() => Session.Dispatch(() =>
        {
            var window = StartGame();

            Assert.Equal(Key.F5, Item(window, "SaveStateMenuItem").Shortcut!.Key);
            Assert.Equal(Key.F8, Item(window, "LoadStateMenuItem").Shortcut!.Key);
            Assert.Equal(Key.P, Item(window, "PauseMenuItem").Shortcut!.Key);
            Assert.Equal(Key.F11, Item(window, "FullscreenMenuItem").Shortcut!.Key);

            window.Close();
        }, default);

        // Showing a gesture is not binding it. MenuBar.SetMenus draws the key and binds
        // nothing, so HotkeyBindingMap stays the only thing that dispatches it - a second
        // binding is how a menu ends up advertising a key a rebind has moved. See §4.19.
        [Fact]
        public Task Advertising_a_key_does_not_bind_a_second_handler_for_it() => Session.Dispatch(() =>
        {
            var window = StartGame();

            Assert.Empty(window.KeyBindings);

            window.Close();
        }, default);

        // Leaving full screen must put back the state it came from - see §4.19.
        [Fact]
        public Task Leaving_fullscreen_returns_a_maximized_window_to_maximized() => Session.Dispatch(() =>
        {
            var window = StartGame();
            window.WindowState = WindowState.Maximized;

            Click(window, "FullscreenMenuItem");
            Assert.Equal(WindowState.FullScreen, window.WindowState);

            Click(window, "FullscreenMenuItem");
            Assert.Equal(WindowState.Maximized, window.WindowState);

            window.Close();
        }, default);

        // The tick follows the window, not the last time a menu was opened - see §4.19.
        [Fact]
        public Task The_fullscreen_tick_follows_a_change_this_window_did_not_make() => Session.Dispatch(() =>
        {
            var window = StartGame();

            window.WindowState = WindowState.FullScreen;

            Assert.True(Item(window, "FullscreenMenuItem").IsChecked);

            window.Close();
        }, default);

        [Fact]
        public Task The_hardware_dashboard_needs_a_debug_target() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            Assert.False(Item(window, "HardwareDashboardMenuItem").IsEnabled);

            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Assert.True(Item(window, "HardwareDashboardMenuItem").IsEnabled);

            window.Close();
        }, default);

        private static System.Collections.Generic.List<LunaAction> SlotItems(MainWindow w) =>
            Item(w, "SaveSlotMenuItem").Submenu!.Items.ToList();

        private static void SelectSlot(MainWindow w, int slot) => SlotItems(w)[slot - 1].Invoke();

        // Every action the window built, by the name its MenuItem used to carry.
        private static System.Collections.Generic.Dictionary<string, LunaAction> Actions(MainWindow w)
        {
            var by = new System.Collections.Generic.Dictionary<string, LunaAction>();
            foreach (var (field, name) in new[]
            {
                ("_pause", "PauseMenuItem"), ("_reset", "ResetMenuItem"), ("_closeGame", "CloseGameMenuItem"),
                ("_saveState", "SaveStateMenuItem"), ("_loadState", "LoadStateMenuItem"),
                ("_speedMenu", "SpeedMenuItem"), ("_slotMenu", "SaveSlotMenuItem"),
                ("_fullscreen", "FullscreenMenuItem"), ("_hardwareDashboard", "HardwareDashboardMenuItem"),
            })
            {
                by[name] = (LunaAction)Field(w, field)!;
            }

            var speeds = (ActionGroup)Field(w, "_speeds")!;
            foreach (var (index, name) in new[]
            {
                (0, "SpeedNormalMenuItem"), (1, "SpeedFastMenuItem"), (2, "SpeedSlowMenuItem"), (3, "SpeedUnthrottledMenuItem"),
            })
            {
                by[name] = speeds.Members[index];
            }

            return by;
        }

        // Private by design - the emulation menu is internal to MainWindow, not API.
        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static object? Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);
    }
}
