using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // U6's wiring step: Controllers ▸ Profiles and a game's own profile, the bindings page, Rumble, Sound ▸ Output, Firmware ▸ Open replacement and the need grouping - see EmuSen_BigPicture.md §52.7.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class SettingsWiringTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(SettingsWiringTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSettingsWiring", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public SettingsWiringTests()
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

        private (MainWindow Window, PadDriver Pad) Open(Action<AppSettings>? settings = null, string? game = null)
        {
            File.WriteAllBytes(Path.Combine(_romDir, game ?? "Granite Choir.sfc"), SyntheticRom.BuildBlank());
            var app = new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever };
            settings?.Invoke(app);
            app.Save();
            var window = new MainWindow { Width = 1280, Height = 800 }.WithoutBattery();
            window.Show();
            var pad = new PadDriver(window) { Pad = { Name = "First Pad" } };
            Dispatcher.UIThread.RunJobs();
            return (window, pad);
        }

        private static SettingsScreen Model(MainWindow w) => w.SettingsShown!;

        private static string ConsoleOf(SettingsScreen s) => s.Host.Systems.First(f => f.Id == "snes").Console;

        // A name typed into the prompt Settings opened, and accepted.
        private static void Answer(MainWindow w, string text)
        {
            Dispatcher.UIThread.RunJobs();
            SheetLayer sheets = w.GetControl<SheetLayer>("Sheets");
            Assert.Equal("PromptWindow", sheets.Current!.GetType().Name);
            Control sheet = sheets.SheetOf(sheets.Current!)!;
            sheet.GetVisualDescendants().OfType<TextBox>().Single(t => t.Name == "PART_Answer").Text = text;
            sheet.GetVisualDescendants().OfType<Button>().Single(b => b.Name == "PART_Accept").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Dispatcher.UIThread.RunJobs();
        }

        private static string Status(SettingsScreen s, string key) => s.Rows.Single(r => r.Key == key).Status ?? "";

        [Fact]
        public Task Profiles_are_made_chosen_renamed_and_deleted_through_Endymions_store_and_saved() => Session.Dispatch(() =>
        {
            (MainWindow window, _) = Open(app => app.BigScreen = true);
            window.ShowSettings(SettingsScreen.Controllers);
            SettingsScreen s = Model(window);
            Assert.Equal(GamepadBindings.DefaultProfile, Status(s, "profiles"));
            s.Choose("profiles");
            Assert.Equal("Profiles", s.Trail[^1]);
            while (s.Rows.Single(r => r.Key == "pconsole").Status != "Super Nintendo") s.Step("pconsole", 1);
            string snes = ConsoleOf(s);
            Assert.Equal(["pconsole", "profile:Default", "newprofile"], s.Rows.Select(r => r.Key));
            Assert.Equal("● Starts games", Status(s, "profile:Default"));

            s.Choose("newprofile");
            Answer(window, "Couch");
            Assert.Equal([GamepadBindings.DefaultProfile, "Couch"], s.Host.PadBindings.Profiles(snes));
            Assert.Contains("profile:Couch", s.Rows.Select(r => r.Key));

            // Chosen for the console: saved in gamepadprofiles.json and read back by a fresh store.
            s.Choose("profile:Couch");
            s.Choose("pdefault");
            Assert.Equal("Couch", GamepadBindings.Load([snes]).ConsoleDefault(snes));
            Assert.Equal("● Chosen", Status(s, "pdefault"));

            // Renamed: the choice follows the name; Default refuses both.
            s.Choose("prename");
            Answer(window, "Sofa");
            Assert.Equal("Sofa", s.Trail[^1]);
            Assert.Equal("Sofa", GamepadBindings.Load([snes]).ConsoleDefault(snes));

            // Deleted: the console starts with Default again, and the page closes back to the list.
            s.Choose("pdelete");
            Assert.Equal("Profiles", s.Trail[^1]);
            GamepadBindings saved = GamepadBindings.Load([snes]);
            Assert.Equal([GamepadBindings.DefaultProfile], saved.Profiles(snes));
            Assert.Equal(GamepadBindings.DefaultProfile, saved.ConsoleDefault(snes));
            s.Choose("profile:Default");
            Assert.False(s.Rows.Single(r => r.Key == "prename").IsEnabled);
            Assert.False(s.Rows.Single(r => r.Key == "pdelete").IsEnabled);
            window.Close();
        }, default);

        [Fact]
        public Task A_games_own_profile_is_a_Games_entry_of_the_store_and_its_session_starts_with_that_profiles_bindings() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Open();
            var bindings = (GamepadBindings)typeof(MainWindow).GetField("_gamepadBindings", Hidden)!.GetValue(window)!;
            string game = Path.Combine(_romDir, "Granite Choir.sfc");
            string snes = RuntimeCatalog.ConsoleForRom(game)!;
            var page = (IGamePageHost)window;
            Assert.Null(page.Profiles(game));

            Assert.True(bindings.CreateProfile(snes, "Couch"));
            bindings.Own(snes, 1, "Couch").Rebind(PadButton.B, SDL.GamepadButton.North);
            bindings.Save();
            Assert.Equal(["Console's (Default)", "Default", "Couch"], page.Profiles(game)!.Select(c => c.Text));
            page.SetProfile(game, "Couch");
            Assert.Equal("Couch", page.Profile(game));
            using (JsonDocument file = JsonDocument.Parse(File.ReadAllText(Path.Combine(ConfigStore.Directory, "gamepadprofiles.json"))))
                Assert.Equal("Couch", file.RootElement.GetProperty(snes).GetProperty("Games").GetProperty("Granite Choir").GetString());
            Assert.Equal("Couch", GamepadBindings.Load([snes]).GameProfile(snes, "Granite Choir"));

            // The session the page's game opens is played with the game's profile: its B on North.
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (clock.ElapsedMilliseconds < 300) { Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(5); }
            pad.Pad.Press(SDL.GamepadButton.South);
            pad.Tick();
            pad.Pad.Release(SDL.GamepadButton.South);
            pad.Tick();
            Assert.NotNull(typeof(MainWindow).GetField("_session", Hidden)!.GetValue(window));
            var gamepad = (GamepadManager)typeof(MainWindow).GetField("_gamepad", Hidden)!.GetValue(window)!;
            Assert.Same(bindings.For(snes, 1, "Couch"), gamepad.Bindings);
            Assert.Equal(SDL.GamepadButton.North, gamepad.Bindings.ButtonToPad[PadButton.B]);

            // Forgotten, the game follows the console's default again.
            page.SetProfile(game, null);
            Assert.Null(GamepadBindings.Load([snes]).GameProfile(snes, "Granite Choir"));
            typeof(MainWindow).GetMethod("StopEmulationThread", Hidden)!.Invoke(window, null);
            window.Close();
        }, default);

        [Fact]
        public Task The_bindings_page_draws_the_console_and_rebinds_the_focused_button_in_its_profile_from_a_press_and_Y_resets_it() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Open(app => app.BigScreen = true);
            window.ShowSettings(SettingsScreen.Controllers);
            SettingsScreen s = Model(window);
            string snes = ConsoleOf(s);
            s.Host.PadBindings.CreateProfile(snes, "Couch");
            s.Host.PadBindings.ChooseDefault(snes, "Couch");

            window.ShowSettings(SettingsScreen.Controllers, "bindings");
            s = Model(window);
            Assert.Equal(["Controllers", "Controller bindings"], s.Trail);
            s.ChooseBindingConsole(snes);
            BindingsPage page = s.Binding!;
            Assert.Equal(snes, page.Console);
            Assert.Equal("Couch", page.Profile);
            SettingsLayout layout = window.SettingsLayoutShown!;
            ((SettingsView)typeof(MainWindow).GetField("_settings", Hidden)!.GetValue(window)!).Fill();
            Assert.NotNull(layout.Content);
            Assert.IsType<ControllerDiagram>(layout.Content!.Body);
            Assert.Equal("SUPER NINTENDO", layout.Content.Tabs[layout.Content.Chosen].ToUpperInvariant());
            Assert.Equal(["Rebind", "Region", "Reset", "Console", "Player", "Test", "Back"], layout.Hints.Select(h => h.Label));

            // Focus Y, the mockup's region; the pane names its binding.
            s.FocusBinding(PadControl.Y);
            ((SettingsView)typeof(MainWindow).GetField("_settings", Hidden)!.GetValue(window)!).Fill();
            Assert.Equal("Bound to the pad's West button.", s.PaneOf(layout.Content!.Focus)!.Lines[2].Text);

            // A listens: the press that chose is let go, then North binds Y in Couch, saved at once; Default is untouched.
            pad.Pad.Press(SDL.GamepadButton.South);
            pad.Tick();
            Assert.True(s.Binding!.Capturing);
            pad.Pad.Release(SDL.GamepadButton.South);
            pad.Tick();
            pad.Pad.Press(SDL.GamepadButton.North);
            pad.Tick();
            Assert.False(s.Binding!.Capturing);
            Assert.Equal(SDL.GamepadButton.North, GamepadBindings.Load([snes]).For(snes, 1, "Couch").ButtonToPad[PadButton.Y]);
            Assert.Equal(SDL.GamepadButton.West, GamepadBindings.Load([snes]).For(snes, 1, GamepadBindings.DefaultProfile).ButtonToPad[PadButton.Y]);
            // The bound press is not the screen's: letting it go starts nothing.
            pad.Pad.Release(SDL.GamepadButton.North);
            pad.Tick();
            Assert.False(s.Binding!.Capturing);

            // Y resets the focused button to its default.
            pad.Tap(SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.West, GamepadBindings.Load([snes]).For(snes, 1, "Couch").ButtonToPad[PadButton.Y]);

            // The shoulders step the console; B closes the page back to its row.
            pad.Tap(SDL.GamepadButton.RightShoulder);
            Assert.NotEqual(snes, s.Binding!.Console);
            pad.Tap(SDL.GamepadButton.East);
            Assert.Null(s.Binding);
            Assert.Null(layout.Content);
            Assert.Equal("bindings", layout.Focused);
            window.Close();
        }, default);

        [Fact]
        public Task A_binding_left_unanswered_gives_up_and_leaves_the_button_as_it_was() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Open(app => app.BigScreen = true);
            window.BindCaptureTimeout = TimeSpan.Zero;
            window.ShowSettings(SettingsScreen.Controllers, "bindings");
            SettingsScreen s = Model(window);
            string console = s.Binding!.Console;
            s.FocusBinding(PadControl.B);
            s.Choose("bind:B");
            pad.Tick();
            pad.Tick();
            Assert.False(s.Binding!.Capturing);
            Assert.Equal(SDL.GamepadButton.South, s.Host.PadBindings.For(console, 1).ButtonToPad[PadButton.B]);
            window.Close();
        }, default);

        [Fact]
        public Task Rumble_is_a_row_where_a_console_drives_a_motor_and_its_switch_is_saved_and_reaches_a_running_game() => Session.Dispatch(() =>
        {
            (MainWindow window, _) = Open();
            window.ShowSettings(SettingsScreen.Controllers);
            SettingsScreen s = Model(window);
            Assert.Contains(s.Host.Systems, f => f.Rumble && f.Id == "n64");
            Assert.DoesNotContain(s.Host.Systems, f => f.Rumble && f.Id == "snes");
            Assert.Equal(SettingsScreen.On, Status(s, "rumble"));
            s.Step("rumble", 1);
            Assert.False(AppSettings.Load().Rumble);
            window.Close();
        }, default);

        [Fact]
        public Task The_frame_loop_hands_each_ports_motors_to_the_players_pad_and_a_stop_once() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = Open();
            var ports = (PortRouter)typeof(MainWindow).GetField("_ports", Hidden)!.GetValue(window)!;
            ports.Reset(2, []);
            (ushort, ushort)[] levels = [(40000, 7), (0, 0)];
            MethodInfo pass = typeof(MainWindow).GetMethod("PassRumbleLevels", Hidden)!;
            pass.Invoke(window, [new Func<int, (ushort, ushort)>(p => levels[p])]);
            Assert.Equal(((ushort)40000, (ushort)7), pad.Pad.Rumble);
            levels[0] = (0, 0);
            pass.Invoke(window, [new Func<int, (ushort, ushort)>(p => levels[p])]);
            Assert.Equal(((ushort)0, (ushort)0), pad.Pad.Rumble);
            window.Close();
        }, default);

        [Fact]
        public Task Sound_output_lists_the_system_default_first_keeps_a_device_not_connected_and_the_players_follow_it() => Session.Dispatch(() =>
        {
            (MainWindow window, _) = Open(app => app.SoundOutputDevice = "Headphones Not Here");
            window.ShowSettings(SettingsScreen.Sound);
            SettingsScreen s = Model(window);
            PageRow output = s.Rows.Single(r => r.Key == "output");
            Assert.Equal(AppSettings.SystemDefaultOutput, output.Choices![0]);
            Assert.Equal("Headphones Not Here", output.Status);
            Assert.Equal(SettingsTone.Attention, s.PaneOf("output")!.Lines[0].Tone);

            // Stepping to the system default saves it, and the game's player plays on it.
            while (s.Rows.Single(r => r.Key == "output").Status != AppSettings.SystemDefaultOutput) s.Step("output", 1);
            Assert.Equal(AppSettings.SystemDefaultOutput, AppSettings.Load().SoundOutputDevice);
            var audio = (EmuSen.Endymion.AudioPlayer)typeof(MainWindow).GetField("_audioPlayer", Hidden)!.GetValue(window)!;
            Assert.Equal(AppSettings.SystemDefaultOutput, audio.OutputDevice);
            window.Close();
        }, default);

        [Fact]
        public Task Open_replacement_is_a_row_of_the_Firmware_page_and_neither_of_the_console_page_nor_its_facts() => Session.Dispatch(() =>
        {
            (MainWindow window, _) = Open();
            window.ShowSettings(SettingsScreen.Consoles);
            SettingsScreen s = Model(window);
            string psx = s.Host.Systems.First(f => f.Id == "psx").Console;
            Assert.Contains(s.Host.Schema(psx), r => r.Key == SettingsScreen.FirmwarePreferenceKey);
            string schemaLabel = s.Host.Schema(psx).Single(r => r.Key == SettingsScreen.FirmwarePreferenceKey).Label;
            Assert.DoesNotContain(schemaLabel + " ", s.PaneOf("console:" + psx)!.Lines.Select(l => l.Text).Aggregate("", (a, b) => a + b));
            s.Choose("console:" + psx);
            Assert.DoesNotContain(s.Rows, r => r.Key == SettingsScreen.FirmwarePreferenceKey);
            s.Choose("firmware");
            Assert.Equal(["fw:PlayStation BIOS", "fwreplacement", "fwfolder", "fwcheck"], s.Rows.Select(r => r.Key));
            Assert.Equal("Open replacement", s.Rows.Single(r => r.Key == "fwreplacement").Label);
            Assert.Equal("In use when none is found", Status(s, "fwreplacement"));
            s.Step("fwreplacement", 1);
            Assert.Equal("Always", Status(s, "fwreplacement"));
            Assert.Equal("replacement", s.Host.CoreValue(psx, SettingsScreen.FirmwarePreferenceKey));
            s.Step("fwreplacement", 1);
            Assert.Equal("own", s.Host.CoreValue(psx, SettingsScreen.FirmwarePreferenceKey));
            window.Close();
        }, default);
    }
}
