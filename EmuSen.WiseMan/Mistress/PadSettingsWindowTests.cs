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
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
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
            OwnLook.Play(window, pad);
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

        public const string Shaders = "Shaders";

        // Up from the top wraps to the bottom, so any entry is found by walking down from the first; the Shaders window's door is Settings ▸ Display's row, reached from the card's Shader Levers.
        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            if (entry != Shaders)
            {
                PadMenu.Choose(window, pad, entry);
                return;
            }
            PadMenu.Choose(window, pad, "Shader Levers");
            pad.B();
            Assert.Equal(SettingsScreen.Display, window.SettingsShown!.Section);
            Assert.Empty(window.SettingsShown.Trail);
            SettingsPad.To(window, pad, "shaders");
            pad.A();
        }

        // B from a sheet the Pause Card opened climbs back to its page, through Settings where Settings opened it, each B a level more, the game paused until the card closes - see EmuSen_Interface.md §2.9 and EmuSen_LibretroHost.md §10z.
        private static void BackToTheGame(MainWindow window, PadDriver pad)
        {
            for (int level = 0; level < 4 && window.SettingsShown is not null; level++)
            {
                Assert.True(IsPaused(window));
                pad.B();
            }
            Assert.Null(window.SettingsShown);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible, "B from the sheet climbs back to the Pause Card");
            for (int level = 0; level < 4 && window.GetControl<Control>("PadMenuPanel").IsVisible; level++)
            {
                Assert.True(IsPaused(window));
                pad.B();
            }
            Assert.False(window.GetControl<Control>("PadMenuPanel").IsVisible);
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
        [InlineData(Shaders)]
        [InlineData("Controller Bindings")]
        [InlineData(MainWindow.CropThisGame)]
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
            BackToTheGame(window, pad);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // What Graphics and Players & Controllers held is Settings' own rows: the card's doors open Settings over the paused game, and every row of the page is reached by the pad - see EmuSen_BigPicture.md §52.5.
        [Theory]
        [InlineData("Console Settings")]
        [InlineData("Shader Levers")]
        [InlineData("Players & Controllers")]
        public Task Every_row_of_each_settings_page_the_card_opens_is_reached_by_the_pad(string entry) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, entry);

            SettingsLayout layout = window.SettingsLayoutShown!;
            Assert.NotNull(layout);
            Assert.NotEmpty(window.SettingsShown!.Trail);
            Assert.False(Sheets(window).IsPresenting);
            Assert.Empty(window.OwnedWindows);
            Assert.True(IsPaused(window));
            Assert.Empty(UnreachedRows(window, pad));

            BackToTheGame(window, pad);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // Down from the first row to the last: the enabled rows the focus never reached, by key.
        private static List<string> UnreachedRows(MainWindow window, PadDriver pad)
        {
            SettingsLayout layout = window.SettingsLayoutShown!;
            var seen = new HashSet<string> { layout.Focused! };
            for (int i = 0; i < layout.Rows.Count; i++) { pad.Down(); seen.Add(layout.Focused!); }
            return layout.Rows.Where(r => r.IsEnabled && !seen.Contains(r.Key)).Select(r => r.Key).ToList();
        }

        // A console's page on Settings, from the card's Console Settings and the shoulders, as a player steps from the running game's console to another.
        private static void ConsolePage(MainWindow window, PadDriver pad, string console)
        {
            Choose(window, pad, "Console Settings");
            StepTo(window, pad, console);
        }

        private static string NameOf(string console) =>
            EmuSen.Mistress.Library.RuntimeCatalog.Systems.First(s => s.Console == console) is var facts && facts.DisplayName is { Length: > 0 } shown ? shown : facts.Name;

        private static void StepTo(MainWindow window, PadDriver pad, string console)
        {
            string name = NameOf(console);
            for (int i = 0; i < 24 && window.SettingsShown!.Trail[^1] != name; i++) pad.R1();
            Assert.Equal(name, window.SettingsShown!.Trail[^1]);
        }

        private static TabControl Tabs(MainWindow w) => Sheet(w).GetVisualDescendants().OfType<TabControl>().First();

        private static void TabTo(MainWindow w, PadDriver pad, string header)
        {
            for (int i = 0; i < 16 && (Tabs(w).SelectedItem as TabItem)?.Header as string != header; i++) pad.R1();
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

        private static string? Stored(string console, string key) => ConsolesConfig.Load().Value(console, key) ?? GraphicsConfig.Load().Value(console, key);

        // A dropdown's list on a sheet, which asks for it inside the window (§4.45.8): the pad moves the focus among its items, never the page behind; the Controller Bindings sheet's player selector - see EmuSen_Settings_Reference.md §4.45.9.
        [Fact]
        public Task An_open_dropdown_drawn_in_the_window_moves_its_highlight_and_not_the_page() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Controller Bindings");
            TabTo(window, pad, "SNES");
            var players = (Dropdown)Reach(window, pad, e => e is Dropdown { Name: "PlayerSelector" });
            ScrollViewer? page = players.FindAncestorOfType<ScrollViewer>();
            Vector scrolled = page?.Offset ?? default;
            int was = players.SelectedIndex;
            Assert.True(players.ItemCount >= 2);

            pad.A();
            Assert.True(players.IsDropDownOpen);
            Assert.True(players.GetVisualDescendants().OfType<Avalonia.Controls.Primitives.Popup>().Single().ShouldUseOverlayLayer);
            Assert.Same(players.ContainerFromIndex(was), Focused(window));
            pad.Down();
            Assert.Same(players.ContainerFromIndex(was + 1), Focused(window));
            pad.Down(players.ItemCount);
            Assert.Same(players.ContainerFromIndex(players.ItemCount - 1), Focused(window));
            Assert.Equal(was, players.SelectedIndex);
            Assert.Equal(scrolled, page?.Offset ?? default);
            Picture(window, "dropdown-in-window");

            pad.B();
            Assert.False(players.IsDropDownOpen);
            Assert.Equal(was, players.SelectedIndex);
            Assert.Same(players, Focused(window));
            Assert.True(Sheets(window).IsPresenting);

            pad.A();
            pad.Down();
            pad.A();
            Assert.False(players.IsDropDownOpen);
            Assert.Equal(was + 1, players.SelectedIndex);
            Assert.Equal($"Player {was + 2}", players.SelectedItem as string);
            Assert.Equal(scrolled, page?.Offset ?? default);
            Stop(window);
            window.Close();
        }, default);

        // Graphics' rows on a console's page in Settings: the shoulders step the console, a choice is stepped both ways and chosen with A, a switch turned, and the console reset - see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task Console_settings_by_pad_the_console_stepped_a_choice_stepped_and_chosen_a_switch_and_reset() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Console Settings");
            Assert.Equal(NameOf("SNES"), window.SettingsShown!.Trail[^1]);
            Picture(window, "console-snes");

            StepTo(window, pad, "N64");
            SettingRow scaleRow = window.SettingsShown!.Host.Schema("N64").Single(r => r.Key == "RenderScale");
            SettingsPad.To(window, pad, "RenderScale");
            int was = scaleRow.Choices.ToList().IndexOf(Stored("N64", "RenderScale") ?? scaleRow.Default);
            pad.Right();
            Assert.Equal(scaleRow.Choices[was + 1], Stored("N64", "RenderScale"));
            pad.Left();
            Assert.Equal(scaleRow.Choices[was], Stored("N64", "RenderScale"));

            // A on a choice steps it on, and the focus and the page stay.
            pad.A();
            Assert.Equal(scaleRow.Choices[was + 1], Stored("N64", "RenderScale"));
            Assert.Equal("RenderScale", window.SettingsLayoutShown!.Focused);
            Assert.Equal(NameOf("N64"), window.SettingsShown.Trail[^1]);

            SettingRow switchRow = window.SettingsShown.Host.Schema("N64").First(r => r.Kind == SettingRow.Switch && !r.Locked && window.SettingsLayoutShown.Rows.Any(p => p.Key == r.Key));
            SettingsPad.To(window, pad, switchRow.Key);
            bool before = (Stored("N64", switchRow.Key) ?? switchRow.Default) is "true" or "1" or "on";
            pad.A();
            Assert.Equal((!before).ToString().ToLowerInvariant(), Stored("N64", switchRow.Key));
            Picture(window, "console-n64");

            SettingsPad.To(window, pad, "reset");
            pad.A();
            Assert.Null(Stored("N64", switchRow.Key));
            Assert.Null(Stored("N64", "RenderScale"));

            BackToTheGame(window, pad);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // Preferences' rows in Settings: a switch in one section and a choice on a page of another, each stored as it is turned - see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task Settings_by_pad_a_switch_and_a_choice_on_a_page_of_another_section() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, "Players & Controllers");
            Assert.Equal("Players", window.SettingsShown!.Trail[^1]);
            Picture(window, "settings-players");
            pad.B();
            Assert.Empty(window.SettingsShown.Trail);

            SettingsPad.Section(window, pad, SettingsScreen.EmuSen);
            SettingsPad.To(window, pad, "background");
            bool before = AppSettings.Load().PauseInBackground;
            pad.A();
            Assert.Equal(!before, AppSettings.Load().PauseInBackground);

            SettingsPad.To(window, pad, "saves");
            pad.A();
            SettingsPad.To(window, pad, "resume");
            string resume = AppSettings.Load().ResumeOnLaunch;
            pad.Left();
            Assert.NotEqual(resume, AppSettings.Load().ResumeOnLaunch);
            Picture(window, "settings-saves");

            // B closes the page for its section, and B again closes Settings onto the card.
            pad.B();
            Assert.NotNull(window.SettingsShown);
            Assert.Empty(window.SettingsShown!.Trail);
            BackToTheGame(window, pad);
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
            OwnLook.Play(window, pad);
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

        // The Shaders window opened from Settings ▸ Display's row, over Settings, searched with the on-screen keyboard, a preset used, a slider moved and reset, and left by pad, the row then naming the shader in use - see EmuSen_Settings_Reference.md §4.48.5.
        [Fact]
        public Task The_shaders_window_opens_over_settings_and_is_searched_used_and_adjusted_by_pad() => Session.Dispatch(() =>
        {
            ShaderSettingsWindowTests.FakePack();
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, Shaders);

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
            Assert.False(Sheets(window).IsPresenting);
            Assert.Equal("shaders", window.SettingsLayoutShown!.Focused);
            Assert.Equal("crt-royale-kurozumi (RetroArch, crt)", SettingsPad.Value(window, "shaders"));
            BackToTheGame(window, pad);
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
            Assert.Equal(EmuSen.Serenity.PictureCrop.None, frame.Crop);
            ((GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).SetValue("SNES", GraphicsSettingsWindow.OverscanKey, GraphicsSettingsWindow.TvOverscan);
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
            Assert.Equal(EmuSen.Serenity.PictureCrop.Television, frame.Crop);
            Assert.Equal(0.4f, frame.ShaderParameters!["maskDark"], 4);

            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            BackToTheGame(window, pad);
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

        // A console's page walked to its foot, stepped to the next console and back with the shoulders: the pad starts at a row in view - see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task A_console_page_stepped_away_from_and_back_to_starts_at_a_row_in_view() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            ConsolePage(window, pad, "N64");
            SettingsLayout layout = window.SettingsLayoutShown!;
            pad.Down(layout.Rows.Count);
            window.UpdateLayout();
            // The big screen builds only the rows in view, so a row in view is one whose row is drawn.
            bool Drawn(string? key) => layout.GetVisualDescendants().OfType<Control>().Any(c => c.Name == "SettingsRow." + key && c.IsEffectivelyVisible);
            Assert.False(Drawn(layout.Rows[0].Key), "the N64 page does not scroll at 1280x800");
            Assert.True(Drawn(layout.Focused));

            pad.R1();
            pad.L1();
            StepTo(window, pad, "N64");
            window.UpdateLayout();
            _out.WriteLine($"focus after the console change: {layout.Focused}");
            Assert.True(Drawn(layout.Focused), $"the focus is on {layout.Focused}, a row out of view");
            BackToTheGame(window, pad);
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

        // Desktop Mode's Settings window, where the retired Debug Logging window's rows are EmuSen ▸ Logging: every row reached through the router, Back closing the page and then the window.
        [Fact]
        public Task In_desktop_mode_the_settings_window_s_logging_page_is_driven_and_every_row_reached() => Session.Dispatch(() =>
        {
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            window.ShowSettings(SettingsScreen.EmuSen, "logging");
            Window other = Assert.Single(window.OwnedWindows);
            Assert.IsType<SettingsWindow>(other);
            Assert.False(Sheets(window).IsPresenting);
            Assert.Equal("Logging", window.SettingsShown!.Trail[^1]);
            void Press(UiButton b) => PadWindowRouter.Send(other, b);

            SettingsLayout layout = window.SettingsLayoutShown!;
            var seen = new HashSet<string?> { layout.Focused };
            for (int i = 0; i <= layout.Rows.Count; i++) { Press(UiButton.Down); seen.Add(layout.Focused); }
            Assert.Empty(layout.Rows.Where(r => r.IsEnabled && !seen.Contains(r.Key)).Select(r => r.Key));

            Press(UiButton.Back);
            Assert.True(other.IsVisible);
            Assert.Empty(window.SettingsShown!.Trail);
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
            Assert.Equal(503, other.Bounds.Height);

            var heights = new List<double>();
            PadAudit.Reachable(other, b => { heights.Add(other.Bounds.Height); PadWindowRouter.Send(other, b); }, limit: 4);
            Assert.NotEmpty(heights);
            Assert.All(heights, h => Assert.Equal(720, h));
            window.Close();
        }, default);

        // The four numbers of a custom crop are a row a pad must reach like any other - see EmuSen_Settings_Reference.md §4.99.
        [Fact]
        public Task With_a_custom_crop_chosen_its_four_numbers_are_reached_by_the_pad() => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            ((GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).SetValue("NES", GraphicsSettingsWindow.OverscanKey, GraphicsSettingsWindow.CustomCrop);
            typeof(MainWindow).GetMethod("ShowGraphicsSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Window other = Assert.Single(window.OwnedWindows);
            void Press(UiButton b) => PadWindowRouter.Send(other, b);

            Press(UiButton.Down);
            HashSet<InputElement> reached = PadAudit.Reachable(other, Press);
            List<InputElement> operable = PadAudit.Operable(other);
            Assert.Equal(new[] { "NES.CropBottom", "NES.CropLeft", "NES.CropRight", "NES.CropTop" }, operable.OfType<TextBox>().Select(b => b.Name).Order());
            Assert.Empty(operable.Where(c => !reached.Contains(c)).Select(PadAudit.Describe));
            window.Close();
        }, default);

        // The explicit exception to a console's games being drawn alike: a crop of the running game's own, from the pad menu, replacing the console's as it is typed - see EmuSen_Settings_Reference.md §4.100.
        [Fact]
        public Task A_crop_of_the_running_game_s_own_replaces_its_console_s_and_graphics_settings_says_so() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            var frame = window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");
            var graphics = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            var records = (EmuSen.Mistress.Library.GameRecords)typeof(MainWindow).GetField("_records", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            string rom = Path.Combine(_romDir, "Game.sfc");
            graphics.SetValue("SNES", GraphicsSettingsWindow.OverscanKey, GraphicsSettingsWindow.TvOverscan);

            Choose(window, pad, MainWindow.CropThisGame);
            var crop = Assert.IsType<GameCropWindow>(Sheets(window).Current);
            Assert.Empty(window.OwnedWindows);
            T Named<T>(string name) where T : Control => Sheet(window).GetVisualDescendants().OfType<T>().First(c => c.Name == name);

            // Until it has one of its own the boxes show the console's, here a television's 3.3% at each edge.
            Assert.Equal("3.3", Named<TextBox>("GameCrop.CropTop").Text);
            Assert.Contains("the SNES's setting", Named<HintText>("GameCropStatus").Text);
            Assert.False(Named<Button>("GameCropUseConsole").IsEnabled);
            Assert.Null(records.Crop(rom));

            Named<TextBox>("GameCrop.CropTop").Text = "8.7";
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.Equal(new EmuSen.Mistress.Library.GameCrop(3.3, 3.3, 8.7, 3.3), records.Crop(rom));
            Assert.Equal(EmuSen.Serenity.PictureCrop.Percent(3.3, 8.7, 3.3, 3.3), frame.Crop);
            Assert.Equal("This game has a crop of its own.", Named<HintText>("GameCropStatus").Text);
            pad.B();
            Assert.False(Sheets(window).IsPresenting);
            BackToTheGame(window, pad);

            // The console's crop row in Settings says what replaces it; another console's says nothing.
            Choose(window, pad, "Console Settings");
            Assert.Contains(window.SettingsShown!.PaneOf("consolecrop")!.Lines, l => l.Text.StartsWith("Game has a crop of its own", StringComparison.Ordinal));
            StepTo(window, pad, "N64");
            Assert.DoesNotContain(window.SettingsShown!.PaneOf("consolecrop")!.Lines, l => l.Text.Contains("crop of its own", StringComparison.Ordinal));
            BackToTheGame(window, pad);

            // The console's own setting changed meanwhile does not reach the frame while the game has its own.
            graphics.SetValue("SNES", GraphicsSettingsWindow.OverscanKey, GraphicsSettingsWindow.NoOverscan);
            typeof(MainWindow).GetMethod("ApplyScreenFilter", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { "SNES" });
            Assert.Equal(EmuSen.Serenity.PictureCrop.Percent(3.3, 8.7, 3.3, 3.3), frame.Crop);

            Choose(window, pad, MainWindow.CropThisGame);
            crop = Assert.IsType<GameCropWindow>(Sheets(window).Current);
            Assert.Equal("8.7", Named<TextBox>("GameCrop.CropTop").Text);
            Named<Button>("GameCropUseConsole").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.Null(records.Crop(rom));
            Assert.Equal(EmuSen.Serenity.PictureCrop.None, frame.Crop);
            Assert.Equal("0", Named<TextBox>("GameCrop.CropTop").Text);
            pad.B();

            Stop(window);
            window.Close();
        }, default);
    }
}
