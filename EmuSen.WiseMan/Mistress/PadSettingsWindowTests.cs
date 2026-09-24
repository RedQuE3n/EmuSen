using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Every settings window reached from the pad's menu in Game Mode, on a sheet, and every control in it reached and operated by the pad - see EmuSen_Settings_Reference.md §4.45.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadSettingsWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadSettingsWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadSettingsWindowTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadSettingsWindowTests(ITestOutputHelper output)
        {
            _out = output;
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

        // Big screen, as Game Mode starts it, with one synthetic game running.
        private (MainWindow Window, PadDriver Pad) GameModeWithAGame()
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true }.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private static string[] MenuLines(MainWindow w) =>
            w.GetControl<ListBox>("PadMenuList").ItemsSource!.Cast<object>().Select(o => o.ToString()!).ToArray();

        // Up from the top wraps to the bottom, so any entry is found by walking down from the first.
        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            int at = Array.FindIndex(MenuLines(window), l => l.StartsWith(entry, StringComparison.Ordinal));
            Assert.True(at >= 0, $"No '{entry}' in the pad menu: {string.Join(", ", MenuLines(window))}");
            pad.Down(at);
            pad.A();
        }

        private static SheetLayer Sheets(MainWindow w) => w.GetControl<SheetLayer>("Sheets");

        private static Control Sheet(MainWindow w) => Sheets(w).SheetOf(Sheets(w).Current!)!;

        private static InputElement? Focused(MainWindow w) => w.FocusManager!.GetFocusedElement() as InputElement;

        private static bool IsPaused(MainWindow w) => w.IsPaused;

        private static void Stop(MainWindow window) =>
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        // Each tab in turn: what the pad cannot reach, by name.
        private List<string> Unreachable(MainWindow window, PadDriver pad)
        {
            var missing = new List<string>();
            Control sheet = Sheet(window);
            TabControl? tabs = sheet.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
            int pages = tabs?.ItemCount ?? 1;

            for (int page = 0; page < pages; page++)
            {
                if (page > 0) pad.R1();
                window.UpdateLayout();
                string where = tabs is null ? "" : $"[{(tabs.SelectedItem as TabItem)?.Header}] ";
                HashSet<InputElement> reached = PadAudit.Reachable(sheet, pad);
                foreach (InputElement control in PadAudit.Operable(sheet).Where(c => !reached.Contains(c)))
                    missing.Add(where + PadAudit.Describe(control));
            }

            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            return missing;
        }

        [Theory]
        [InlineData("Graphics Settings")]
        [InlineData("Controller Bindings")]
        [InlineData("Preferences")]
        public Task Every_control_of_each_settings_sheet_is_reached_by_the_pad(string entry) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, entry);

            Assert.True(Sheets(window).IsPresenting);
            Assert.Empty(window.OwnedWindows);
            Assert.True(IsPaused(window));

            Assert.Empty(Unreachable(window, pad));

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        private static TabControl Tabs(MainWindow w) => Sheet(w).GetVisualDescendants().OfType<TabControl>().First();

        private static void TabTo(MainWindow w, PadDriver pad, string header)
        {
            for (int i = 0; i < 8 && (Tabs(w).SelectedItem as TabItem)?.Header as string != header; i++) pad.R1();
            Assert.Equal(header, (Tabs(w).SelectedItem as TabItem)?.Header);
        }

        private static InputElement Reach(MainWindow w, PadDriver pad, Func<InputElement, bool> target) => PadAudit.Reach(Sheet(w), pad, target);

        private static void Picture(MainWindow w, string name) => UiTest.Dump("pad-" + name, UiTest.Capture(w));

        private static string? Stored(string console, string key) => GraphicsConfig.Load().Value(console, key);

        [Fact]
        public Task Graphics_by_pad_tabs_a_dropdown_stepped_opened_committed_and_cancelled_a_switch_and_reset() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Graphics Settings");
            Assert.Equal("SNES", (Tabs(window).SelectedItem as TabItem)?.Header);
            Picture(window, "graphics-snes");

            TabTo(window, pad, "N64");
            var engine = (Dropdown)Reach(window, pad, e => e is Dropdown { Name: "N64.Engine" });
            int was = engine.SelectedIndex;
            pad.Right();
            Assert.Equal(was + 1, engine.SelectedIndex);
            Assert.Equal(engine.SelectedItem as string, Stored("N64", "Engine"));
            pad.Left();
            Assert.Equal(was, engine.SelectedIndex);

            // Open, move, commit: the highlight moves the focus, and only A chooses.
            pad.A();
            Assert.True(engine.IsDropDownOpen);
            Picture(window, "graphics-n64-dropdown");
            pad.Down();
            Assert.Equal(was, engine.SelectedIndex);
            pad.A();
            Assert.False(engine.IsDropDownOpen);
            Assert.Equal(was + 1, engine.SelectedIndex);
            Assert.Equal(engine.SelectedItem as string, Stored("N64", "Engine"));

            // Open, move, back out: nothing chosen, and the sheet stays.
            pad.A();
            pad.Up();
            pad.B();
            Assert.False(engine.IsDropDownOpen);
            Assert.Equal(was + 1, engine.SelectedIndex);
            Assert.True(Sheets(window).IsPresenting);
            Assert.Same(engine, Focused(window));

            var toggle = (LunaSwitch)Reach(window, pad, e => e is LunaSwitch);
            string key = toggle.Name!["N64.".Length..];
            bool before = toggle.IsChecked == true;
            pad.A();
            Assert.Equal(!before, toggle.IsChecked == true);
            Assert.Equal((!before).ToString().ToLowerInvariant(), Stored("N64", key));
            Picture(window, "graphics-n64");

            Reach(window, pad, e => e is Button { Content: "Reset This Console" });
            pad.A();
            Assert.Null(Stored("N64", key));
            Assert.Null(Stored("N64", "Engine"));

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task Preferences_by_pad_a_switch_and_a_dropdown_on_another_tab() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Preferences");
            Picture(window, "preferences-library");

            TabTo(window, pad, "Gameplay");
            Reach(window, pad, e => e is LunaSwitch { Name: "PauseInBackgroundSwitch" });
            bool before = AppSettings.Load().PauseInBackground;
            pad.A();
            Assert.Equal(!before, AppSettings.Load().PauseInBackground);

            Reach(window, pad, e => e is Dropdown { Name: "ResumeDropdown" });
            string resume = AppSettings.Load().ResumeOnLaunch;
            pad.Left();
            Assert.NotEqual(resume, AppSettings.Load().ResumeOnLaunch);
            Picture(window, "preferences-gameplay");

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task Controller_bindings_by_pad_the_slider_a_switch_a_pad_rebind_and_a_key_rebind_backed_out_of() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Controller Bindings");
            var bindings = (InputSettingsWindow)Sheets(window).Current!;
            Picture(window, "bindings-snes");

            TabTo(window, pad, "General");
            var slider = (Slider)Reach(window, pad, e => e is Slider);
            double deadzone = AppSettings.Load().StickDeadzone;
            pad.Right();
            Assert.Equal(deadzone + 0.05, AppSettings.Load().StickDeadzone, 3);
            Assert.Same(slider, Focused(window));

            Reach(window, pad, e => e is LunaSwitch { Name: "AnalogStickAsDpadCheckBox" });
            bool dpad = AppSettings.Load().AnalogStickAsDpad;
            pad.A();
            Assert.Equal(!dpad, AppSettings.Load().AnalogStickAsDpad);
            Picture(window, "bindings-general");

            // The A that chose Rebind Pad is not the binding; the next press is, and the pad is the window's again once it is let go.
            TabTo(window, pad, "SNES");
            var rebind = (Button)Reach(window, pad, e => e is Button { Content: "Rebind Pad" });
            var before = new System.Collections.Generic.Dictionary<PadButton, SDL.GamepadButton>(EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad);
            pad.Pad.Press(SDL.GamepadButton.South);
            pad.Tick();
            Assert.Equal(PadCapture.PadButton, bindings.Capturing);
            Poll(bindings);
            Assert.Equal(before, EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad);
            pad.Pad.Release(SDL.GamepadButton.South);
            pad.Tick();
            Poll(bindings);
            pad.Pad.Press(SDL.GamepadButton.North);
            pad.Tick();
            Poll(bindings);
            Assert.Same(rebind, Focused(window));
            Assert.Equal(PadCapture.PadButton, bindings.Capturing);
            pad.Pad.Release(SDL.GamepadButton.North);
            pad.Tick();
            Poll(bindings);
            Assert.Equal(PadCapture.None, bindings.Capturing);
            Assert.Contains(EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad, kv => kv.Value == SDL.GamepadButton.North);

            // A keyboard key cannot come from the pad, so B backs out of the capture and nothing else does.
            Reach(window, pad, e => e is Button { Content: "Rebind Key" });
            pad.A();
            Assert.Equal(PadCapture.Key, bindings.Capturing);
            pad.Down();
            Assert.Equal(PadCapture.Key, bindings.Capturing);
            pad.B();
            Assert.Equal(PadCapture.None, bindings.Capturing);
            Assert.True(Sheets(window).IsPresenting);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        [Fact]
        public Task A_pad_capture_nobody_answers_gives_up() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Controller Bindings");
            var bindings = (InputSettingsWindow)Sheets(window).Current!;
            bindings.PadCaptureTimeout = TimeSpan.Zero;

            Reach(window, pad, e => e is Button { Content: "Rebind Pad" });
            pad.A();
            Poll(bindings);
            Poll(bindings);
            Assert.Equal(PadCapture.None, bindings.Capturing);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        private static void Poll(InputSettingsWindow window) =>
            typeof(InputSettingsWindow).GetMethod("PollForPadButton", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
    }
}
