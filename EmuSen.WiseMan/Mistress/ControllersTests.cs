using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // Every pad steering the library, the menu and a sheet; hot-plug, the notices, the swap, the first controller alone, and every SDL handle let go - see EmuSen_Settings_Reference.md §4.61.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllersTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ControllersTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenControllersTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public ControllersTests()
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

        private (MainWindow Window, PadDriver First) LibraryOf(int games, Action<AppSettings>? settings = null)
        {
            for (int i = 0; i < games; i++)
                File.WriteAllBytes(Path.Combine(_romDir, $"Game {i:00}.sfc"), SyntheticRom.BuildBlank());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true };
            settings?.Invoke(app);
            app.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window) { Pad = { Name = "First Pad" } };
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            return (window, pad);
        }

        private static AppSettings Settings(MainWindow w) => (AppSettings)typeof(MainWindow).GetField("_appSettings", Hidden)!.GetValue(w)!;

        private static int Selected(MainWindow w) => w.GetControl<ListBox>("LibraryList").SelectedIndex;

        private static bool MenuOpen(MainWindow w) => w.GetControl<Control>("PadMenuPanel").IsVisible;

        private static string[] MenuLines(MainWindow w) =>
            w.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        private static void MenuEntry(MainWindow window, PadDriver pad, string entry)
        {
            pad.Start();
            Assert.True(MenuOpen(window));
            int at = Array.FindIndex(MenuLines(window), l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu");
            pad.Down(at);
            pad.A();
        }

        private static T Named<T>(Control root, string name) where T : Control =>
            Avalonia.VisualTree.VisualExtensions.GetVisualDescendants(root).OfType<T>().First(c => c.Name == name);

        private static NoticeLayer Notice(MainWindow w) => w.GetControl<NoticeLayer>("PadNotice");

        // Lets real time pass with the dispatcher's own loop running, so a DispatcherTimer fires (LunaP §88.7).
        private static void Wait(TimeSpan span)
        {
            var frame = new DispatcherFrame();
            var stop = new DispatcherTimer { Interval = span };
            stop.Tick += (_, _) => { stop.Stop(); frame.Continue = false; };
            stop.Start();
            Dispatcher.UIThread.PushFrame(frame);
        }

        [Fact]
        public Task Two_pads_both_steer_the_library_the_menu_and_a_sheet() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(5);
            PadDriver second = first.Plug("Second Pad", SDL.GamepadType.PS4);
            first.Tick();

            first.Down();
            Assert.Equal(1, Selected(window));
            second.Down();
            Assert.Equal(2, Selected(window));
            second.Up();
            first.Down(2);
            Assert.Equal(3, Selected(window));

            second.Start();
            Assert.True(MenuOpen(window));
            first.B();
            Assert.False(MenuOpen(window));

            MenuEntry(window, second, "Preferences");
            Assert.IsType<PreferencesWindow>(window.GetControl<SheetLayer>("Sheets").Current);
            first.B();
            Assert.False(window.GetControl<SheetLayer>("Sheets").IsPresenting);
            window.Close();
        }, default);

        [Fact]
        public Task Unplugging_one_pad_leaves_the_other_steering_and_playing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(4);
            PadDriver second = first.Plug("Second Pad");
            first.Tick();

            first.Unplug();
            first.Tick();
            Assert.Equal("Second Pad", first.Gamepad.ControllerName);

            second.Down(2);
            Assert.Equal(2, Selected(window));

            // The pad left is player 1's now, as the next pad opened was before there were two.
            second.Pad.Press(SDL.GamepadButton.South);
            Assert.True(first.Gamepad.IsPressed(PadButton.B));
            second.Pad.Release(SDL.GamepadButton.South);
            window.Close();
        }, default);

        // The rescan fix of 2026-09-26 (§4.4): a pad lost and found again, with every poll between, throws nothing.
        [Fact]
        public Task A_pad_pulled_out_and_plugged_back_is_picked_up_without_an_error() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(3);

            first.Unplug();
            for (int i = 0; i < 120; i++) first.Tick();
            Assert.False(first.Gamepad.IsConnected);
            StringAssert(window.GetControl<TextBlock>("LibraryHintText").Text, "Double-click a title");

            first.Replug();
            first.Tick();
            Assert.True(first.Gamepad.IsConnected);
            StringAssert(window.GetControl<TextBlock>("LibraryHintText").Text, "A  Play");
            first.Down();
            Assert.Equal(1, Selected(window));
            window.Close();
        }, default);

        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public Task A_notice_says_when_a_pad_connects_and_disconnects_and_then_goes_away(bool bigScreen) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(2, a => a.BigScreen = bigScreen);
            Assert.Equal(bigScreen, window.GetControl<SheetLayer>("Sheets").PresentsWindows);
            first.Tick();
            Assert.Null(Notice(window).Current);

            // ES-DE's recorded popup: about four seconds, half a second to fade in and to fade out.
            Assert.Equal(TimeSpan.FromSeconds(4), Notice(window).Duration);
            Assert.Equal(TimeSpan.FromSeconds(0.5), Notice(window).FadeTime);

            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            Assert.Equal("Controller connected: Second Pad", Notice(window).Current);

            second.Unplug();
            first.Tick();
            Assert.Equal("Controller disconnected: Second Pad", Notice(window).Current);

            Wait(TimeSpan.FromSeconds(4.5));
            Assert.Null(Notice(window).Current);
            window.Close();
        }, default);

        [Fact]
        public Task With_notifications_off_no_notice_is_shown() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(2, a => a.ControllerNotifications = false);
            PadDriver second = first.Plug("Second Pad");
            first.Tick();
            second.Unplug();
            first.Tick();
            Assert.Null(Notice(window).Current);
            window.Close();
        }, default);

        [Fact]
        public Task With_the_first_controller_only_the_second_pad_steers_nothing() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(4, a => a.FirstControllerOnly = true);
            PadDriver second = first.Plug("Second Pad");
            first.Tick();

            second.Down();
            second.Start();
            Assert.Equal(0, Selected(window));
            Assert.False(MenuOpen(window));

            first.Down();
            Assert.Equal(1, Selected(window));
            window.Close();
        }, default);

        // The swap trades Accept and Back, and Search moves to West, in the library, the menu, a sheet and the hints.
        [Fact]
        public Task The_swap_trades_accept_and_back_everywhere_in_the_interface() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(3, a => a.SwapPadButtons = true);
            first.Tick();
            Assert.StartsWith("B  Choose      A  Back", window.GetControl<SheetLayer>("Sheets").Hint);

            first.Start();
            Assert.True(MenuOpen(window));
            first.A();
            Assert.False(MenuOpen(window));
            StringAssert(window.GetControl<TextBlock>("PadMenuHint").Text, "B  Choose      A  Close");

            first.Start();
            int at = Array.FindIndex(MenuLines(window), l => l.StartsWith("Preferences", StringComparison.Ordinal));
            first.Down(at);
            first.B();
            Assert.IsType<PreferencesWindow>(window.GetControl<SheetLayer>("Sheets").Current);
            first.A();
            Assert.False(window.GetControl<SheetLayer>("Sheets").IsPresenting);

            first.Y();
            Assert.Null(OnScreenKeyboard.OpenOver(window));
            first.X();
            StringAssert(OnScreenKeyboard.OpenOver(window)!.Hint, "B  Type      A  Erase      X  Space");
            first.Start();
            Assert.Null(OnScreenKeyboard.OpenOver(window));
            first.A();
            Assert.False(window.GetControl<Control>("LibraryFilter").IsKeyboardFocusWithin);

            first.B();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            window.Close();
        }, default);

        private static void StringAssert(string? actual, string expected) => Assert.True(actual?.StartsWith(expected, StringComparison.Ordinal), $"'{actual}' does not start with '{expected}'");

        // The game keeps its own bindings: South is the SNES B with the swap on or off.
        [Fact]
        public Task The_swap_does_not_reach_the_game() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(1, a => a.SwapPadButtons = true);
            first.B();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);

            var held = (bool[])typeof(MainWindow).GetField("_gamepadHeld", Hidden)!.GetValue(window)!;
            MethodInfo poll = typeof(MainWindow).GetMethod("PollGamepad", Hidden)!;
            for (int i = 0; i < 3; i++) poll.Invoke(window, null);
            first.Pad.Press(SDL.GamepadButton.South);
            poll.Invoke(window, null);
            Assert.True(held[(int)PadButton.B]);
            Assert.False(held[(int)PadButton.A]);
            first.Pad.Release(SDL.GamepadButton.South);
            poll.Invoke(window, null);

            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            window.Close();
        }, default);

        [Fact]
        public Task The_controller_settings_persist() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(1);
            MenuEntry(window, first, "Preferences");
            var prefs = Assert.IsType<PreferencesWindow>(window.GetControl<SheetLayer>("Sheets").Current);
            prefs.ShowTab(PreferencesWindow.ControllersTab);
            window.UpdateLayout();
            Dispatcher.UIThread.RunJobs();

            Named<Dropdown>(window, "ControllerTypeDropdown").SelectedItem = "Nintendo";
            Named<LunaSwitch>(window, "SwapPadButtonsSwitch").IsChecked = true;
            Named<LunaSwitch>(window, "FirstControllerOnlySwitch").IsChecked = true;
            Named<LunaSwitch>(window, "ControllerNotificationsSwitch").IsChecked = false;
            window.Close();

            AppSettings saved = AppSettings.Load();
            Assert.Equal(("Nintendo", true, true, false), (saved.ControllerType, saved.SwapPadButtons, saved.FirstControllerOnly, saved.ControllerNotifications));
            Assert.Equal((AppSettings.ControllerTypeAutomatic, false, false, true),
                (new AppSettings().ControllerType, new AppSettings().SwapPadButtons, new AppSettings().FirstControllerOnly, new AppSettings().ControllerNotifications));
        }, default);

        // Plan §15.14's rule for pads: every SDL handle is let go when its pad goes and when the window closes.
        [Fact]
        public Task Every_pad_s_handle_is_released_when_the_pad_goes_and_when_the_window_closes() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver first) = LibraryOf(1);
            PadDriver second = first.Plug("Second Pad");
            first.Plug("Third Pad");
            first.Tick();
            Assert.Equal(3, first.Devices.OpenHandles);

            second.Unplug();
            first.Tick();
            Assert.Equal(2, first.Devices.OpenHandles);

            window.Close();
            Assert.Equal(0, first.Devices.OpenHandles);
            Assert.False(first.Devices.Initialized);
        }, default);
    }
}
