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
        private (MainWindow Window, PadDriver Pad) GameModeWithAGame()
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = true }.Save();

            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            var pad = new PadDriver(window);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            OwnLook.Play(window, pad);
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);
            return (window, pad);
        }

        private static string[] MenuLines(MainWindow w) => PadMenu.AllLines(w);

        public const string Shaders = "Shaders";

        public const string ConsoleSettings = "Console Settings", ShaderLevers = "Shader Levers";

        // The card opened, then a row of it, or a Settings page at Settings' own door over the card, since the card's Display is its own page (EmuSen_Interface.md §20); Shaders is Display's row.
        private static void Choose(MainWindow window, PadDriver pad, string entry)
        {
            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            string console = (string)typeof(MainWindow).GetField("_activeConsole", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            if (entry == ConsoleSettings) { window.ShowSettings(SettingsScreen.Consoles, "console:" + console); return; }
            if (entry == ShaderLevers) { window.ShowSettings(SettingsScreen.Display, "levers"); return; }
            if (entry != Shaders)
            {
                PadMenu.Choose(window, pad, entry);
                return;
            }
            window.ShowSettings(SettingsScreen.Display, "levers");
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

        // What Graphics held is Settings' own rows: the card's doors open Settings over the paused game, and every row of the page is reached by the pad - see EmuSen_BigPicture.md §52.5.
        [Theory]
        [InlineData(ConsoleSettings)]
        [InlineData(ShaderLevers)]
        [InlineData(Shaders)]
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
            Choose(window, pad, ConsoleSettings);
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

        // Graphics' rows on a console's page in Settings: the shoulders step the console, a choice is stepped both ways and chosen with A, a switch turned, and the console reset - see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task Console_settings_by_pad_the_console_stepped_a_choice_stepped_and_chosen_a_switch_and_reset() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, ConsoleSettings);
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

        // A row of the running game's engine changed in Settings over the game: the page is the running session's own rows, and the change is kept and reaches the session at once - the path the hosted PlayStation 2's Internal Resolution took on the handheld on 2026-11-02, see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task A_running_engine_s_row_changed_in_settings_is_kept_and_reaches_the_session() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            object game = typeof(MainWindow).GetField("_session", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            var crystal = (EmuSen.Crystal.Client.CrystalSession)game.GetType().GetProperty("Crystal")!.GetValue(game)!;
            IReadOnlyList<SettingRow> running = RuntimeCatalog.SettingsOf(crystal);

            Choose(window, pad, ConsoleSettings);
            List<string> keys = window.SettingsLayoutShown!.Rows.Select(r => r.Key).ToList();
            Assert.Contains(keys, k => running.Any(r => r.Key == k));
            SettingRow row = running.First(r => r.Kind == SettingRow.Choice && !r.Locked && r.Choices.Count > 1 && keys.Contains(r.Key));
            string now = ConsolesConfig.Load().Value("SNES", row.Key) ?? row.Default;
            string next = row.Choices[(row.Choices.ToList().IndexOf(now) + 1) % row.Choices.Count];
            SettingsPad.To(window, pad, row.Key);
            pad.Right();
            Assert.Equal(next, ConsolesConfig.Load().Value("SNES", row.Key));
            Assert.Equal(next, crystal.Setting(row.Key, game: false));

            BackToTheGame(window, pad);
            Stop(window);
            window.Close();
        }, default);

        // Preferences' rows in Settings: a switch in one section and a choice on a page of another, each stored as it is turned - see EmuSen_BigPicture.md §52.5.
        [Fact]
        public Task Settings_by_pad_a_switch_and_a_choice_on_a_page_of_another_section() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, ConsoleSettings);
            Assert.Equal("Super Nintendo", window.SettingsShown!.Trail[^1]);
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

        // The bindings page by pad, in Game Mode over the paused game: the deadzone and the stick's switch are Controllers rows, the A that chose a rebind is not the binding, and a key's capture is backed out of with B - see EmuSen_Input.md §10.6.
        [Fact]
        public Task Controller_bindings_by_pad_the_deadzone_a_switch_a_pad_rebind_and_a_key_rebind_backed_out_of() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, MainWindow.ControllersMenu);
            PadMenu.Choose(window, pad, "Bindings", exact: true);
            SettingsScreen settings = window.SettingsShown!;
            Assert.Equal("Controller bindings", settings.Trail[^1]);
            Assert.Equal("SNES", settings.Binding!.Console);
            Picture(window, "bindings-snes");

            // B to Controllers, where the deadzone steps by five and the stick's switch turns.
            pad.B();
            double deadzone = AppSettings.Load().StickDeadzone;
            window.SettingsLayoutShown!.FocusOn("deadzone");
            pad.Right();
            Assert.Equal(deadzone + 0.05, AppSettings.Load().StickDeadzone, 3);
            window.SettingsLayoutShown!.FocusOn("stickdpad");
            bool dpad = AppSettings.Load().AnalogStickAsDpad;
            pad.A();
            Assert.Equal(!dpad, AppSettings.Load().AnalogStickAsDpad);

            // The A that chose the rebind is not the binding; the next press is, and the pad is the page's again once it is let go.
            window.SettingsLayoutShown!.FocusOn("bindings");
            pad.A();
            settings.ChooseBindingConsole("SNES");
            var before = new System.Collections.Generic.Dictionary<PadButton, SDL.GamepadButton>(EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad);
            pad.Pad.Press(SDL.GamepadButton.South);
            pad.Tick();
            Assert.True(settings.Binding!.Capturing);
            Assert.Equal(before, EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad);
            pad.Pad.Release(SDL.GamepadButton.South);
            pad.Tick();
            pad.Pad.Press(SDL.GamepadButton.North);
            pad.Tick();
            Assert.False(settings.Binding!.Capturing);
            pad.Pad.Release(SDL.GamepadButton.North);
            pad.Tick();
            Assert.False(settings.Binding!.Capturing);
            Assert.Contains(EmuSen.Endymion.Input.GamepadBindings.Load(new[] { "SNES" }).For("SNES").ButtonToPad, kv => kv.Value == SDL.GamepadButton.North);

            // A keyboard key cannot come from the pad, so B backs out of its capture and nothing else does.
            pad.B();
            window.SettingsLayoutShown!.FocusOn("keys");
            pad.A();
            Assert.True(settings.Binding!.Keys);
            pad.A();
            Assert.True(settings.Binding!.Capturing);
            pad.Down();
            Assert.True(settings.Binding!.Capturing);
            pad.B();
            Assert.False(settings.Binding!.Capturing);
            Assert.Equal("Keyboard keys", settings.Trail[^1]);

            pad.B();
            pad.B();
            Assert.Null(window.SettingsShown);
            Stop(window);
            window.Close();
        }, default);

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
            // The card slides out before the grid takes the pad again (EmuSen_Interface.md §20.6.1).
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (window.GetControl<Control>("PadMenuPanel").IsVisible && clock.ElapsedMilliseconds < 5000) { pad.Tick(); Avalonia.Threading.Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(5); }

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

        // A shader's page in Settings over the paused game: a preset found by the search typed by pad, used, its parameter stepped and back, and left by pad, Display's row then naming the shader in use - see EmuSen_BigPicture.md §52.9.
        [Fact]
        public Task The_shaders_page_is_searched_used_and_adjusted_by_pad() => Session.Dispatch(() =>
        {
            ShaderSettingsTests.FakePack();
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            Choose(window, pad, Shaders);
            Assert.Equal("Shaders", window.SettingsShown!.Trail[1]);
            Assert.Empty(UnreachedRows(window, pad));

            SettingsPad.To(window, pad, "find");
            pad.A();
            Answer(window, pad, "kuro");
            Assert.Equal(["use:" + EmuSen.Serenity.Shaders.ScreenFilters.None, "use:slang:crt/crt-royale-kurozumi.slangp"], window.SettingsLayoutShown!.Rows.Select(r => r.Key));
            Picture(window, "shaders-search");

            SettingsPad.To(window, pad, "use:slang:crt/crt-royale-kurozumi.slangp");
            pad.A();
            Assert.Equal("slang:crt/crt-royale-kurozumi.slangp", Stored("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter));
            pad.B();
            SettingsPad.To(window, pad, "params");
            pad.A();
            SettingsPad.To(window, pad, "p:SCAN");
            pad.Right();
            Assert.Equal("6", GraphicsConfig.Load().ParametersFor("SNES", "slang:crt/crt-royale-kurozumi.slangp")["SCAN"]);
            Assert.Equal("p:SCAN", window.SettingsLayoutShown!.Focused);
            Picture(window, "shaders-parameter");
            pad.Left();
            Assert.Empty(GraphicsConfig.Load().ParametersFor("SNES", "slang:crt/crt-royale-kurozumi.slangp"));

            // B a page at a time to Display, whose Shaders row names the shader in use.
            pad.B();
            pad.B();
            pad.B();
            Assert.Empty(window.SettingsShown!.Trail);
            Assert.Equal("crt-royale-kurozumi (RetroArch, crt)", SettingsPad.Value(window, "shaders"));
            BackToTheGame(window, pad);
            Stop(window);
            window.Close();
        }, default);

        // The question a row asked on the big screen, typed with the on-screen keyboard and its OK pressed, as a player does.
        private static void Answer(MainWindow window, PadDriver pad, string words)
        {
            Assert.Equal("PromptWindow", Sheets(window).Current!.GetType().Name);
            var box = (TextBox)Reach(window, pad, e => e is TextBox { Name: "PART_Answer" });
            pad.A();
            OnScreenKeyboard keyboard = OnScreenKeyboard.OpenOver(window)!;
            Assert.Same(box, keyboard.Target);
            PadCheatsTests.TypeByPad(pad, keyboard, words);
            pad.Start();
            Reach(window, pad, e => e is Button { Name: "PART_Accept" });
            pad.A();
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Assert.False(Sheets(window).IsPresenting);
        }

        // The SNES's shader page with a shader in use, its parameters page opened by pad.
        private void ParametersOf(MainWindow window, PadDriver pad, string stored)
        {
            Choose(window, pad, Shaders);
            if (stored.StartsWith(EmuSen.Mistress.Library.ShaderCatalog.SlangPrefix, StringComparison.Ordinal))
            {
                SettingsPad.To(window, pad, "folder:" + EmuSen.Mistress.Library.ShaderCatalog.Category(stored[EmuSen.Mistress.Library.ShaderCatalog.SlangPrefix.Length..]));
                pad.A();
                SettingsPad.To(window, pad, "use:" + stored);
                pad.A();
                pad.B();
            }
            else
            {
                SettingsPad.To(window, pad, "use:" + stored);
                pad.A();
            }
            Assert.Equal(stored, Stored("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter));
            SettingsPad.To(window, pad, "params");
            pad.A();
            Assert.Equal("Parameters", window.SettingsShown!.Trail[^1]);
        }

        private static string? RowOf(MainWindow window) => window.SettingsLayoutShown!.Focused;

        // The big screen draws only the rows in view, so a row in view is one whose row is drawn.
        private static bool Drawn(MainWindow window, string? key) => window.SettingsLayoutShown!.GetVisualDescendants().OfType<Control>().Any(c => c.Name == "SettingsRow." + key && c.IsEffectivelyVisible);

        // Pressed straight down from a shader's first parameter, the pad reaches each parameter in turn, the page following it.
        [Fact]
        public Task Down_from_a_shader_s_first_parameter_reaches_every_parameter_in_turn() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            ParametersOf(window, pad, EmuSen.Serenity.Shaders.CrtFilters.Lottes.Name);
            string[] keys = EmuSen.Serenity.Shaders.CrtFilters.LottesParameters.Select(p => "p:" + p.Id).ToArray();
            SettingsPad.To(window, pad, keys[0]);
            foreach (string key in keys.Skip(1))
            {
                pad.Down();
                Assert.Equal(key, RowOf(window));
                Assert.True(Drawn(window, key), $"{key} is focused but not drawn");
            }
            Stop(window);
            window.Close();
        }, default);

        // Straight from the card: a built-in filter used, a parameter moved, Reset all, moved again, and the value drawn with, the console's shape and crop with it - see EmuSen_Settings_Reference.md §4.48.5.
        [Fact]
        public Task The_shaders_page_from_the_card_uses_a_built_in_filter_and_its_parameters_reach_the_frame() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            var frame = window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");
            Assert.False(frame.SquarePixels);
            Assert.Equal(EmuSen.Serenity.PictureCrop.None, frame.Crop);
            ((GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.Overscan, EmuSen.Mistress.Library.GraphicsKeys.TvOverscan);
            ((GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!).SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.PictureShape, EmuSen.Mistress.Library.GraphicsKeys.SquarePixels);
            ParametersOf(window, pad, EmuSen.Serenity.Shaders.CrtFilters.Lottes.Name);
            Assert.True(IsPaused(window));

            SettingsPad.To(window, pad, "p:maskDark");
            pad.Left();
            pad.Left();
            Assert.Equal("0.3", GraphicsConfig.Load().ParametersFor("SNES", "CRT (Lottes)")["maskDark"]);
            Picture(window, "shaders-lottes");

            SettingsPad.To(window, pad, "paramreset");
            pad.A();
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            SettingsPad.To(window, pad, "p:maskDark");
            pad.Left();
            Assert.Equal("CRT (Lottes)", Stored("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter));
            Assert.Equal("CRT (Lottes)", frame.ActiveFilter?.Name);
            Assert.Equal("SNES", frame.FilterConsole);
            Assert.True(frame.SquarePixels, "the console's stored picture shape reaches the frame with its filter");
            Assert.Equal(EmuSen.Serenity.PictureCrop.Television, frame.Crop);
            Assert.Equal(0.4f, frame.ShaderParameters!["maskDark"], 4);

            BackToTheGame(window, pad);
            Assert.False(IsPaused(window));
            Stop(window);
            window.Close();
        }, default);

        // A preset of 944 parameters on the big screen: only the rows in view are drawn, and the pad walks down and moves a parameter far down - see EmuSen_Settings_Reference.md §4.48.9.
        [Fact]
        public Task A_long_preset_s_parameters_are_drawn_a_page_at_a_time_and_walked_by_pad() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            ParametersOf(window, pad, "slang:big/huge.slangp");
            SettingsLayout layout = window.SettingsLayoutShown!;
            Assert.Equal(944, layout.Rows.Where(r => r.Key.StartsWith("sec:", StringComparison.Ordinal)).Sum(r => int.Parse(r.Status!)));
            SettingsPad.To(window, pad, "sec:HEAD0");
            pad.A();
            Assert.Equal(105, layout.Rows.Count(r => r.Key.StartsWith("p:", StringComparison.Ordinal)));
            int drawn() => layout.GetVisualDescendants().OfType<Control>().Count(c => c.Name is { } n && n.StartsWith("SettingsRow.", StringComparison.Ordinal));
            Assert.InRange(drawn(), 1, 40);

            SettingsPad.To(window, pad, "p:P0001");
            pad.Down(30);
            Assert.Equal("p:P0031", RowOf(window));
            Assert.True(Drawn(window, "p:P0031"));
            Assert.InRange(drawn(), 1, 40);
            pad.Right();
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", "slang:big/huge.slangp")["P0031"]);
            Picture(window, "shaders-944-walked");
            Stop(window);
            window.Close();
        }, default);

        // The parameter search on the big screen: reached by pad, typed with the on-screen keyboard, and the one row it leaves moved by pad - see EmuSen_Settings_Reference.md §4.48.10.
        [Fact]
        public Task The_parameter_search_is_reached_and_typed_by_pad() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            ParametersOf(window, pad, "slang:big/huge.slangp");
            SettingsPad.To(window, pad, "paramsearch");
            pad.A();
            Answer(window, pad, "0031");
            Assert.Equal(["p:P0031"], window.SettingsLayoutShown!.Rows.Select(r => r.Key).Where(k => k.StartsWith("p:", StringComparison.Ordinal)));
            Picture(window, "shaders-944-search");
            SettingsPad.To(window, pad, "p:P0031");
            pad.Right();
            Assert.Equal("0.55", GraphicsConfig.Load().ParametersFor("SNES", "slang:big/huge.slangp")["P0031"]);
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
        // Back on a parameters page left part way down, the pad starts at a row in view.
        [Fact]
        public Task A_parameters_page_opened_again_puts_the_focus_on_a_row_in_view() => Session.Dispatch(() =>
        {
            ShaderBrowseTests.WriteBig(ShaderSettingsTests.FakePack());
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            ParametersOf(window, pad, "slang:big/huge.slangp");
            SettingsPad.To(window, pad, "sec:HEAD0");
            pad.A();
            SettingsPad.To(window, pad, "p:P0001");
            pad.Down(40);
            Assert.False(Drawn(window, "p:P0001"), "the page did not move down");
            Assert.True(Drawn(window, RowOf(window)));

            pad.B();
            Assert.Equal("sec:HEAD0", RowOf(window));
            pad.A();
            _out.WriteLine($"focus on opening again: {RowOf(window)}");
            Assert.True(Drawn(window, RowOf(window)), $"the focus is on {RowOf(window)}, a row out of view");
            Stop(window);
            window.Close();
        }, default);

        // Desktop Mode keeps real windows; the headless platform never makes one active, so the router is handed the window the pad would have found.
        [Theory]
        [InlineData("ShowActiveCheats")]
        [InlineData("ShowThemeSettings")]
        public Task In_desktop_mode_a_real_window_is_driven_and_every_control_reached(string opener) => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
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
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
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
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            typeof(MainWindow).GetMethod("ShowThemeSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Window other = Assert.Single(window.OwnedWindows);

            PadWindowRouter.Send(other, UiButton.Down);
            PadWindowRouter.Send(other, UiButton.PageDown);
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            double grown = other.Bounds.Height;

            var heights = new List<double>();
            PadAudit.Reachable(other, b => { heights.Add(other.Bounds.Height); PadWindowRouter.Send(other, b); }, limit: 4);
            Assert.NotEmpty(heights);
            Assert.All(heights, h => Assert.Equal(grown, h));
            window.Close();
        }, default);

        // The four numbers of a custom crop are rows a pad must reach like any other - see EmuSen_Settings_Reference.md §4.99.
        [Fact]
        public Task With_a_custom_crop_chosen_its_four_numbers_are_reached_by_the_pad() => Session.Dispatch(() =>
        {
            new AppSettings { BigPictureInterface = { MenuOpeningEffect = BigPictureInterface.OpeningNone }, RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            var graphics = new GraphicsConfig();
            graphics.SetValue("NES", EmuSen.Mistress.Library.GraphicsKeys.Overscan, EmuSen.Mistress.Library.GraphicsKeys.CustomCrop);
            graphics.Save();
            MainWindow window = SettingsDesk.Open(SettingsScreen.Consoles, "console:NES", "consolecropedges");
            Assert.Equal(["crop:CropLeft", "crop:CropRight", "crop:CropTop", "crop:CropBottom"], SettingsDesk.Keys(window));
            var seen = new HashSet<string?> { SettingsDesk.Layout(window).Focused };
            for (int i = 0; i < 4; i++) { SettingsDesk.Press(window, UiButton.Down); seen.Add(SettingsDesk.Layout(window).Focused); }
            Assert.Empty(SettingsDesk.Keys(window).Where(k => !seen.Contains(k)));
            window.Close();
        }, default);

        // The explicit exception to a console's games being drawn alike: a crop of the running game's own, from the Pause Card's Display, replacing the console's edge by edge - see EmuSen_Interface.md §20.
        [Fact]
        public Task A_crop_of_the_running_game_s_own_replaces_its_console_s_and_graphics_settings_says_so() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = GameModeWithAGame();
            var frame = window.GetControl<EmuSen.Serenity.GameFrameControl>("GameFrame");
            var graphics = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            var records = (EmuSen.Mistress.Library.GameRecords)typeof(MainWindow).GetField("_records", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            string rom = Path.Combine(_romDir, "Game.sfc");
            graphics.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.Overscan, EmuSen.Mistress.Library.GraphicsKeys.TvOverscan);

            // Game Options ▸ Display ▸ This game's crop: Right turns it on from the console's, a television's 3.3% at each edge, and A opens its edges.
            void Display()
            {
                pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                PadMenu.Choose(window, pad, MainWindow.GameOptionsMenu, exact: true);
                PadMenu.Choose(window, pad, MainWindow.DisplayMenu, exact: true);
                pad.Down(System.Array.IndexOf(PadMenu.Lines(window), MainWindow.CropThisGame));
            }
            Display();
            Assert.Equal("Off", PadMenu.Entries(window)[PadMenu.Selected(window)].Value!());
            Assert.Null(records.Crop(rom));
            pad.Right();
            Assert.Equal(new EmuSen.Mistress.Library.GameCrop(3.3, 3.3, 3.3, 3.3), records.Crop(rom));
            pad.A();
            Assert.Equal(MainWindow.CropThisGame, PadMenu.PageTitle(window));
            Assert.Equal(["This game's crop", "Left", "Right", "Top", "Bottom"], PadMenu.Entries(window).Select(e => e.Label!()));
            pad.Down(3);
            for (int i = 0; i < 54; i++) pad.Right();
            Assert.Equal(new EmuSen.Mistress.Library.GameCrop(3.3, 3.3, 8.7, 3.3), records.Crop(rom));
            Assert.Equal(EmuSen.Serenity.PictureCrop.Percent(3.3, 8.7, 3.3, 3.3), frame.Crop);
            Assert.True(IsPaused(window));
            pad.Start();

            // The console's crop row in Settings says what replaces it; another console's says nothing.
            Choose(window, pad, ConsoleSettings);
            Assert.Contains(window.SettingsShown!.PaneOf("consolecrop")!.Lines, l => l.Text.StartsWith("Game has a crop of its own", StringComparison.Ordinal));
            StepTo(window, pad, "N64");
            Assert.DoesNotContain(window.SettingsShown!.PaneOf("consolecrop")!.Lines, l => l.Text.Contains("crop of its own", StringComparison.Ordinal));
            BackToTheGame(window, pad);

            // The console's own setting changed meanwhile does not reach the frame while the game has its own.
            graphics.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.Overscan, EmuSen.Mistress.Library.GraphicsKeys.NoOverscan);
            typeof(MainWindow).GetMethod("ApplyScreenFilter", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { "SNES" });
            Assert.Equal(EmuSen.Serenity.PictureCrop.Percent(3.3, 8.7, 3.3, 3.3), frame.Crop);

            Display();
            Assert.Equal("On", PadMenu.Entries(window)[PadMenu.Selected(window)].Value!());
            pad.Left();
            Assert.Null(records.Crop(rom));
            Assert.Equal(EmuSen.Serenity.PictureCrop.None, frame.Crop);
            pad.Start();

            Stop(window);
            window.Close();
        }, default);
    }
}
