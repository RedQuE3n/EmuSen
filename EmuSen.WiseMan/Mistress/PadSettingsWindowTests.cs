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
        private (MainWindow Window, PadDriver Pad) GameModeWithAGame(double tallerBar = 0)
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true }.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            if (tallerBar > 0) TallerBar(window, tallerBar);
            return (window, pad);
        }

        // The status bar made taller by some pixels, so a sheet above it is shorter by as much - see EmuSen_Settings_Reference.md §4.83.7.
        private static void TallerBar(MainWindow window, double by)
        {
            var bar = window.GetControl<Border>("StatusBar");
            window.UpdateLayout();
            bar.MinHeight = bar.Bounds.Height + by;
            window.UpdateLayout();
        }

        // Sheets shorter by these pixels: 0 to 7 left a row unbuilt, 3 also a view that would not scroll, 7 and 8 bracket the first big-screen status line, 10 and 13 failed the walk's replay (§4.83.7).
        public static TheoryData<double> Shorter => new() { 0, 3, 5, 7, 8, 10, 13 };

        public static TheoryData<double> EveryShorter => new(Enumerable.Range(0, 17).Select(h => (double)h));

        public static TheoryData<double> MuchShorter => new() { 60, 61, 62, 66, 70, 74, 75, 80 };

        private static string[] MenuLines(MainWindow w) => PadMenu.AllLines(w);

        // Up from the top wraps to the bottom, so any entry is found by walking down from the first.
        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            PadMenu.Choose(window, pad, entry);
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
        [InlineData("Graphics")]
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

            // B a screen at a time: Preferences, ES-DE's menus in Game Mode, goes back from its last submenu first (§4.72.8).
            for (int guard = 0; guard < 3 && Sheets(window).IsPresenting; guard++) pad.B();
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

        // Presses Up until the focus is on the control asked for, failing after as many presses as a long column could need.
        private static void UpTo(MainWindow w, PadDriver pad, Func<InputElement, bool> target)
        {
            for (int i = 0; i < 60 && !(Focused(w) is { } f && target(f)); i++) pad.Up();
            Assert.True(Focused(w) is { } at && target(at), $"Up never reached the control asked for; the focus is on {(Focused(w) is { } e ? PadAudit.Describe(e) : "nothing")}.");
        }

        private static void Picture(MainWindow w, string name) => UiTest.Dump("pad-" + name, UiTest.Capture(w));

        private static string? Stored(string console, string key) => GraphicsConfig.Load().Value(console, key);

        // A dropdown's list on a sheet, which asks for it inside the window (§4.45.8): the pad moves the focus among its items, never the page behind - see EmuSen_Settings_Reference.md §4.45.9.
        [Fact]
        public Task An_open_dropdown_drawn_in_the_window_moves_its_highlight_and_not_the_page() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Graphics");
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
            Choose(window, pad, "Graphics");
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

            // In Game Mode Preferences is ES-DE's menus: B leaves Gameplay for the first screen, and B again closes it (§4.72.8).
            pad.B();
            Assert.True(Sheets(window).IsPresenting);
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
            Choose(window, pad, "Quit Game");
            PadMenu.Answer(window, pad, yes: true);
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
            Choose(window, pad, "Graphics");
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

        // Pressed straight down from a preset's first slider, with no walk, the pad reaches each row's Reset and then its slider, whatever the sheet's height - see EmuSen_Settings_Reference.md §4.83.7.
        [Theory]
        [MemberData(nameof(EveryShorter))]
        public Task Down_from_a_preset_s_first_slider_reaches_every_slider_at_any_sheet_height(double tallerBar) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame(tallerBar);
            Choose(window, pad, "Shaders");
            ShaderPanel snes = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current).PanelFor("SNES");
            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "CRT (Lottes)");
            string? RowOf() => (Focused(window) as Visual)?.FindAncestorOfType<SliderRow>()?.Label;

            pad.Right();
            Pump(snes);
            string[] labels = snes.Parameters.Select(p => p.Label).ToArray();
            for (int i = 0; i < 4 && !(Focused(window) is Slider && RowOf() == labels[0]); i++) pad.Down();
            Assert.Equal(labels[0], RowOf());
            Assert.IsType<Slider>(Focused(window));

            // The view moves by no more than a row a press, so building the next row never jumps the page.
            ScrollViewer view = snes.ParameterList.FindDescendantOfType<ScrollViewer>()!;
            double pitch = snes.Sliders.First().Bounds.Height + snes.Sliders.First().Margin.Bottom;
            void Press()
            {
                double was = view.Offset.Y;
                pad.Down();
                Assert.True(System.Math.Abs(view.Offset.Y - was) <= pitch, $"the view moved {view.Offset.Y - was:F0} for one press, more than a row ({pitch:F0})");
            }
            foreach (string label in labels.Skip(1))
            {
                Press();
                Assert.True(Focused(window) is Button && RowOf() == label, $"Down went to {PadAudit.Describe(Focused(window)!)}, not the Reset of {label}");
                Press();
                Assert.True(Focused(window) is Slider && RowOf() == label, $"Down went to {PadAudit.Describe(Focused(window)!)}, not the slider of {label}");
            }
            Stop(window);
            window.Close();
        }, default);

        // Straight from the pad's menu: a built-in filter's slider moved, Reset All, moved again, then the filter used and the value drawn with - see EmuSen_Settings_Reference.md §4.48.5.
        [Theory]
        [MemberData(nameof(Shorter))]
        public Task The_shaders_window_from_the_pad_menu_adjusts_a_built_in_filter_and_resets_it_all(double tallerBar) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame(tallerBar);
            Choose(window, pad, "Shaders");
            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            Assert.True(IsPaused(window));
            ShaderPanel snes = shaders.PanelFor("SNES");
            var frame = window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");

            Assert.False(frame.SquarePixels);
            ((GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).SetValue("SNES", GraphicsSettingsWindow.PictureShapeKey, GraphicsSettingsWindow.SquarePixels);
            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "CRT (Lottes)");
            Assert.Equal("CRT (Lottes)", snes.Shown?.Stored);
            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Mask dark" });
            pad.Left();
            pad.Left();
            Assert.Equal("0.3", GraphicsConfig.Load().ParametersFor("SNES", "CRT (Lottes)")["maskDark"]);
            Assert.Null(frame.ActiveFilter);
            Picture(window, "shaders-lottes");

            // Up the column of sliders to the row of Reset All and Use: the route a player takes, pressed rather than searched for, since a search that wanders onto the list changes the shader shown - see EmuSen_Settings_Reference.md §4.96.
            UpTo(window, pad, e => e is Button { Name: "SNES.UseShader" or "SNES.ResetShader" });
            if ((Focused(window) as Control)?.Name == "SNES.UseShader") pad.Left();
            Assert.Equal("SNES.ResetShader", (Focused(window) as Control)?.Name);
            pad.A();
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            Assert.IsType<Button>(Focused(window));
            Assert.All(snes.Sliders, s => Assert.True(s.IsDefault));

            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Mask dark" });
            pad.Left();
            UpTo(window, pad, e => e is Button { Name: "SNES.UseShader" or "SNES.ResetShader" });
            if ((Focused(window) as Control)?.Name == "SNES.ResetShader") pad.Right();
            Assert.Equal("SNES.UseShader", (Focused(window) as Control)?.Name);
            pad.A();
            Assert.IsType<Button>(Focused(window));
            Assert.Equal("CRT (Lottes)", Stored("SNES", GraphicsSettingsWindow.ScreenFilterKey));
            Assert.Equal("CRT (Lottes)", frame.ActiveFilter?.Name);
            Assert.Equal("SNES", frame.FilterConsole);
            Assert.True(frame.SquarePixels, "the console's stored picture shape reaches the frame with its filter");
            Assert.Equal(0.4f, frame.ShaderParameters!["maskDark"], 4);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // A preset of 944 parameters on the Game Mode sheet: only the rows in view are built, every control is still reached, and the pad walks down and moves a slider far down - see EmuSen_Settings_Reference.md §4.48.9.
        [Theory]
        [MemberData(nameof(Shorter))]
        public Task A_long_preset_s_sliders_on_the_sheet_are_reached_and_walked_by_pad(double tallerBar) => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsWindowTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame(tallerBar);
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

        // The same walk under a status bar two lines taller, where a first search from the category found no path from 61 to 74 and at 80 pixels shorter - see VenusRT_Native.md §66.
        [Theory]
        [MemberData(nameof(MuchShorter))]
        public Task A_long_preset_s_sliders_are_reached_under_a_much_taller_status_bar(double tallerBar) => A_long_preset_s_sliders_on_the_sheet_are_reached_and_walked_by_pad(tallerBar);

        // The parameter search on the sheet: reached by pad, typed with the on-screen keyboard, and the one row it leaves moved by pad - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public Task The_parameter_search_is_reached_and_typed_by_pad_on_the_sheet() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsWindowTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Shaders");
            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            ShaderPanel snes = shaders.PanelFor("SNES");
            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "huge");
            Pump(snes);

            // The walk to the box may pass over other rows of the shader list, so the preset is chosen again after typing.
            var search = (TextBox)Reach(window, pad, e => e is TextBox && e.FindAncestorOfType<FilterBar>() is { Name: "SNES.ParameterSearch" });
            pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(window)!;
            Assert.Same(search, keyboard.Target);
            PadCheatsTests.TypeByPad(pad, keyboard, "0031");
            pad.Start();
            // A row reached sideways from the search box takes the focus but not the choice, so the pad steps off it and back, as a player would (§4.83.7, Q194).
            var huge = (ListBoxItem)Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "huge");
            if (!huge.IsSelected) { pad.Up(); pad.Down(); }
            Assert.True(Focused(window) is ListBoxItem { IsSelected: true } chosen && chosen.Content?.ToString() == "huge");
            Pump(snes);
            Assert.Equal("0031", snes.ParameterList.Search);
            Assert.Equal(new[] { "Parameter 0031" }, snes.ParameterList.Matching.Select(p => p.Label));
            Picture(window, "shaders-944-search");

            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Parameter 0031" });
            pad.Right();
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", "slang:big/huge.slangp")["P0031"]);
            pad.B();
            Stop(window);
            window.Close();
        }, default);

        // A Graphics tab scrolled to its foot, left and come back to: the pad starts at a control in view and the page stays where it was - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public Task A_tab_change_back_to_a_scrolled_page_starts_at_a_control_in_view() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Graphics");
            TabTo(window, pad, "N64");
            ScrollViewer page = (ScrollViewer)((Avalonia.Controls.Presenters.ContentPresenter)Tabs(window).GetVisualDescendants().OfType<Avalonia.Controls.Presenters.ContentPresenter>().First(p => p.Name == "PART_SelectedContentHost")).Content!;
            InputElement last = PadAudit.Operable(page).OrderBy(e => ((Visual)e).TranslatePoint(default, page)?.Y ?? 0).Last();
            Reach(window, pad, e => ReferenceEquals(e, last));
            window.UpdateLayout();
            Vector scrolled = page.Offset;
            Assert.True(scrolled.Y > 0, "the N64 page does not scroll at 1280x800");

            TabTo(window, pad, "NES");
            TabTo(window, pad, "N64");
            var focused = Assert.IsAssignableFrom<Control>(Focused(window));
            _out.WriteLine($"focus after the tab change: {PadAudit.Describe((InputElement)focused)}, page at {page.Offset.Y} of {scrolled.Y}");
            Assert.Equal(scrolled, page.Offset);
            Assert.True(focused.TranslatePoint(default, page) is { } at && at.Y + focused.Bounds.Height > 0 && at.Y < page.Bounds.Height);
            pad.B();
            pad.B();
            Stop(window);
            window.Close();
        }, default);

        // Back on a tab whose parameter list is scrolled part way down, the pad starts at a control in view, and the list stays where it was - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public Task A_tab_change_never_puts_the_focus_on_a_row_scrolled_out_of_view() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsWindowTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Shaders");
            var shaders = Assert.IsType<ShaderSettingsWindow>(Sheets(window).Current);
            ShaderPanel snes = shaders.PanelFor("SNES");
            Reach(window, pad, e => e is ListBoxItem item && item.Content?.ToString() == "huge");
            Pump(snes);
            Reach(window, pad, e => e is Slider s && s.FindAncestorOfType<SliderRow>() is { Label: "Parameter 0001" });
            pad.Down(2 * 20);
            ScrollViewer scroll = snes.ParameterList.GetVisualDescendants().OfType<ScrollViewer>().First();
            Vector scrolled = scroll.Offset;
            Assert.True(scrolled.Y > 0);
            Assert.Contains(snes.Sliders, r => r.TranslatePoint(default, scroll) is { Y: < 0 } at && at.Y + r.Bounds.Height <= 0);

            TabTo(window, pad, "N64");
            TabTo(window, pad, "SNES");
            var focused = Assert.IsAssignableFrom<Control>(Focused(window));
            _out.WriteLine($"focus after the tab change: {PadAudit.Describe((InputElement)focused)}");
            Assert.Null(focused.FindAncestorOfType<SliderList>());
            Assert.Equal(scrolled, scroll.Offset);
            Picture(window, "shaders-tab-return");

            // Up to the tab strip and down into the page again, the page now laid out with the rows above the view built.
            for (int i = 0; i < 6 && Focused(window) is not TabItem; i++) pad.Up();
            Assert.IsType<TabItem>(Focused(window));
            window.UpdateLayout();
            Assert.Contains(snes.Sliders, r => r.TranslatePoint(default, scroll) is { } at && at.Y + r.Bounds.Height <= 0);
            pad.Down();
            focused = Assert.IsAssignableFrom<Control>(Focused(window));
            _out.WriteLine($"focus down from the tab strip: {PadAudit.Describe((InputElement)focused)}");
            Assert.Null(focused.FindAncestorOfType<SliderList>());
            Assert.Equal(scrolled, scroll.Offset);
            pad.B();
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
                HashSet<InputElement> reached = PadAudit.Reachable(other, Press);
                missing.AddRange(PadAudit.Operable(other).Where(c => !reached.Contains(c)).Select(c => $"[{(tabs?.SelectedItem as TabItem)?.Header}] {PadAudit.Describe(c)}"));
            }
            foreach (string m in missing) _out.WriteLine("unreachable: " + m);
            Assert.Empty(missing);

            Press(UiButton.Back);
            Assert.False(other.IsVisible);
            window.Close();
        }, default);

        // A window that sizes to its tab grows through the dispatcher, so the search runs it before its first press too - see EmuSen_Settings_Reference.md §4.98.
        [Fact]
        public Task The_desktop_search_presses_only_in_a_window_grown_to_its_tab() => Session.Dispatch(() =>
        {
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            typeof(MainWindow).GetMethod("ShowPreferences", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Window other = Assert.Single(window.OwnedWindows);

            PadWindowRouter.Send(other, UiButton.Down);
            PadWindowRouter.Send(other, UiButton.PageDown);
            Assert.Equal(531, other.Bounds.Height);

            var heights = new List<double>();
            PadAudit.Reachable(other, b => { heights.Add(other.Bounds.Height); PadWindowRouter.Send(other, b); }, limit: 4);
            Assert.NotEmpty(heights);
            Assert.All(heights, h => Assert.Equal(720, h));
            window.Close();
        }, default);
    }
}
