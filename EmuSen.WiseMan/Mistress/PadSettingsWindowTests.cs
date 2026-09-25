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
        [InlineData("Shaders")]
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

        // A dropdown's list on a sheet, which asks for it inside the window (§4.45.8): the pad moves the focus among its items, never the page behind - see EmuSen_Settings_Reference.md §4.45.9.
        [Fact]
        public Task An_open_dropdown_drawn_in_the_window_moves_its_highlight_and_not_the_page() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Graphics Settings");
            TabTo(window, pad, "N64");
            var engine = (Dropdown)Reach(window, pad, e => e is Dropdown { Name: "N64.Engine" });
            ScrollViewer page = engine.FindAncestorOfType<ScrollViewer>()!;
            Vector scrolled = page.Offset;
            int was = engine.SelectedIndex;
            Assert.True(engine.ItemCount >= 2);

            pad.A();
            Assert.True(engine.IsDropDownOpen);
            Assert.True(engine.GetVisualDescendants().OfType<Avalonia.Controls.Primitives.Popup>().Single().ShouldUseOverlayLayer);
            Assert.Same(engine.ContainerFromIndex(was), Focused(window));
            pad.Down();
            Assert.Same(engine.ContainerFromIndex(was + 1), Focused(window));
            pad.Down(engine.ItemCount);
            Assert.Same(engine.ContainerFromIndex(engine.ItemCount - 1), Focused(window));
            Assert.Equal(was, engine.SelectedIndex);
            Assert.Equal(scrolled, page.Offset);
            Picture(window, "dropdown-in-window");

            pad.B();
            Assert.False(engine.IsDropDownOpen);
            Assert.Equal(was, engine.SelectedIndex);
            Assert.Same(engine, Focused(window));

            pad.A();
            pad.Down();
            pad.A();
            Assert.False(engine.IsDropDownOpen);
            Assert.Equal(was + 1, engine.SelectedIndex);
            Assert.Equal(engine.SelectedItem as string, Stored("N64", "Engine"));
            Assert.Equal(scrolled, page.Offset);
            Stop(window);
            window.Close();
        }, default);

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

        // The question a game with a resume state asks as it starts: on a sheet in Game Mode, answered with A, or B to not start at all - see EmuSen_Settings_Reference.md §4.45.2.
        [Theory]
        [InlineData(true)]
        [InlineData(false)]
        public Task The_resume_question_is_asked_on_a_sheet_and_answered_by_pad(bool resume) => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings
            {
                RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeAsk, BigScreen = true,
                StateDirectory = Path.Combine(_root, "States"), LogDirectory = Path.Combine(_root, "Logs"),
            }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Choose(window, pad, "Close Game");
            Assert.True(window.GetControl<Control>("LibraryView").IsVisible);

            pad.A();
            var ask = Assert.IsType<ResumeWindow>(Sheets(window).Current);
            Assert.Empty(window.OwnedWindows);
            Assert.Equal("ResumeButton", (Focused(window) as Control)?.Name);
            Picture(window, "resume");

            if (resume) pad.A(); else pad.B();

            Assert.False(Sheets(window).IsPresenting);
            Assert.Equal(resume, window.GetControl<Control>("GameFrame").IsVisible);
            Assert.Equal(!resume, window.GetControl<Control>("LibraryView").IsVisible);
            Stop(window);
            window.Close();
        }, default);

        // The Shaders window opened from the graphics sheet's row, over it, searched with the on-screen keyboard, a preset used, a slider moved and reset, and left by pad - see EmuSen_Settings_Reference.md §4.48.5.
        [Fact]
        public Task The_shaders_window_opens_over_the_graphics_sheet_and_is_searched_used_and_adjusted_by_pad() => Session.Dispatch(() =>
        {
            ShaderSettingsWindowTests.FakePack();
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Graphics Settings");
            var graphics = Sheets(window).Current;
            Reach(window, pad, e => e is Button { Name: "SNES.Shaders" });
            pad.A();

            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            Assert.Equal("SNES", (Tabs(window).SelectedItem as TabItem)?.Header);
            Assert.Empty(Unreachable(window, pad));
            TabTo(window, pad, "SNES");

            var search = (TextBox)Reach(window, pad, e => e is TextBox && e.FindAncestorOfType<FilterBar>() is { Name: "SNES.ShaderSearch" });
            pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(window)!;
            Assert.Same(search, keyboard.Target);
            PadCheatsTests.TypeByPad(pad, keyboard, "kuro");
            pad.Start();
            ShaderPanel snes = shaders.PanelFor("SNES");
            Assert.Equal(new[] { "None", "crt-royale-kurozumi" }, snes.List.Models.Select(e => e.Name));
            Picture(window, "shaders-search");

            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "crt-royale-kurozumi");
            Pump(snes);
            pad.A();
            Assert.Equal("slang:crt/crt-royale-kurozumi.slangp", Stored("SNES", GraphicsSettingsWindow.ScreenFilterKey));

            var slider = (Slider)Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Scanline weight" });
            pad.Right();
            Assert.Equal("6", GraphicsConfig.Load().ParametersFor("SNES", "slang:crt/crt-royale-kurozumi.slangp")["SCAN"]);
            Assert.Same(slider, Focused(window));
            Picture(window, "shaders-slider");
            pad.Up();
            Assert.Equal("Reset Scanline weight", Avalonia.Automation.AutomationProperties.GetName((Focused(window) as Control)!));
            pad.A();
            Assert.Empty(GraphicsConfig.Load().ParametersFor("SNES", "slang:crt/crt-royale-kurozumi.slangp"));

            pad.B();
            Assert.Same(graphics, Sheets(window).Current);
            Assert.Equal("crt-royale-kurozumi (RetroArch, crt)", Sheet(window).GetVisualDescendants().OfType<TextBlock>().Single(t => t.Name == "SNES.ShaderInUse").Text);
            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        // Straight from the pad's menu: a built-in filter's slider moved, Reset All, moved again, then the filter used and the value drawn with - see EmuSen_Settings_Reference.md §4.48.5.
        [Fact]
        public Task The_shaders_window_from_the_pad_menu_adjusts_a_built_in_filter_and_resets_it_all() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Shaders");
            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            Assert.True(IsPaused(window));
            ShaderPanel snes = shaders.PanelFor("SNES");
            var frame = window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");

            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "CRT (Lottes)");
            Assert.Equal("CRT (Lottes)", snes.Shown?.Stored);
            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Mask dark" });
            pad.Left();
            pad.Left();
            Assert.Equal("0.3", GraphicsConfig.Load().ParametersFor("SNES", "CRT (Lottes)")["maskDark"]);
            Assert.Null(frame.ActiveFilter);
            Picture(window, "shaders-lottes");

            Reach(window, pad, e => e is Button { Name: "SNES.ResetShader" });
            pad.A();
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            Assert.IsType<Button>(Focused(window));
            Assert.All(snes.Sliders, s => Assert.True(s.IsDefault));

            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Mask dark" });
            pad.Left();
            Reach(window, pad, e => e is Button { Name: "SNES.UseShader" });
            pad.A();
            Assert.IsType<Button>(Focused(window));
            Assert.Equal("CRT (Lottes)", Stored("SNES", GraphicsSettingsWindow.ScreenFilterKey));
            Assert.Equal("CRT (Lottes)", frame.ActiveFilter?.Name);
            Assert.Equal(0.4f, frame.ShaderParameters!["maskDark"], 4);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // A preset of 944 parameters on the Game Mode sheet: only the rows in view are built, every control is still reached, and the pad walks down and moves a slider far down - see EmuSen_Settings_Reference.md §4.48.9.
        [Fact]
        public Task A_long_preset_s_sliders_on_the_sheet_are_reached_and_walked_by_pad() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsWindowTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Shaders");
            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            ShaderPanel snes = shaders.PanelFor("SNES");

            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "huge");
            Pump(snes);
            window.UpdateLayout();
            Assert.Equal(944, snes.Parameters.Count);
            Assert.InRange(snes.Sliders.Count(), 1, 40);
            Assert.Empty(Unreachable(window, pad));
            TabTo(window, pad, "SNES");
            Assert.Equal("huge", snes.Shown?.Name);

            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Parameter 0001" });
            pad.Down(2 * 30);
            Assert.Equal("Parameter 0031", (Focused(window) as Visual)?.FindAncestorOfType<SliderRow>()?.Label);
            Assert.IsType<Slider>(Focused(window));
            Assert.InRange(snes.Sliders.Count(), 1, 40);
            pad.Right();
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", "slang:big/huge.slangp")["P0031"]);
            Picture(window, "shaders-944-walked");

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Stop(window);
            window.Close();
        }, default);

        private static void Pump(ShaderPanel panel)
        {
            for (int i = 0; i < 2000 && !panel.Reading.IsCompleted; i++) { Avalonia.Threading.Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(1); }
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
        }

        // Desktop Mode keeps real windows; the headless platform never makes one active, so the router is handed the window the pad would have found.
        [Theory]
        [InlineData("ShowDebugLogging")]
        [InlineData("ShowGraphicsSettings")]
        [InlineData("ShowShaderSettings")]
        [InlineData("ShowActiveCheats")]
        [InlineData("ShowPreferences")]
        public Task In_desktop_mode_a_real_window_is_driven_and_every_control_reached(string opener) => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();

            typeof(MainWindow).GetMethod(opener, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Window other = Assert.Single(window.OwnedWindows);
            Assert.False(Sheets(window).IsPresenting);
            void Press(UiButton b) => PadWindowRouter.Send(other, b);

            var missing = new List<string>();
            TabControl? tabs = other.GetVisualDescendants().OfType<TabControl>().FirstOrDefault();
            Press(UiButton.Down);
            for (int page = 0; page < (tabs?.ItemCount ?? 1); page++)
            {
                if (page > 0) Press(UiButton.PageDown);
                other.UpdateLayout();
                HashSet<InputElement> reached = PadAudit.Reachable(other, Press);
                missing.AddRange(PadAudit.Operable(other).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs?.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);

            Press(UiButton.Back);
            Assert.False(other.IsVisible);
            window.Close();
        }, default);
    }
}
