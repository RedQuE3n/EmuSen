using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.WiseMan.LunaP;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.Interactivity;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The library screen and the switch to the game screen - see EmuSen_Settings_Reference.md §4.11.
    public class MainWindowLibraryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowLibraryTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;

        public MainWindowLibraryTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenMainWindowLibraryTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // MainWindow reads AppSettings itself, so the setting has to be on
        // disk before it is constructed.
        private void ConfigureRomDirectory(string? directory)
        {
            new AppSettings { RomDirectory = directory }.Save();
        }

        private void WriteRom(string fileName) =>
            File.WriteAllBytes(Path.Combine(_romDir, fileName), SyntheticRom.BuildBlank());

        private static ListBox LibraryList(MainWindow w) => w.GetControl<ListBox>("LibraryList");
        private static Control LibraryView(MainWindow w) => w.GetControl<DockPanel>("LibraryView");
        private static Control GameFrame(MainWindow w) => w.GetControl<Control>("GameFrame");
        private static TextBlock Header(MainWindow w) => w.GetControl<TextBlock>("LibraryHeaderText");

        private static List<string> Titles(MainWindow w) =>
            LibraryList(w).ItemsSource!.Cast<string>().ToList();

        [Fact]
        public Task A_configured_directory_lists_its_games_on_the_library_screen() => Session.Dispatch(() =>
        {
            WriteRom("Zelda.smc");
            WriteRom("Actraiser.sfc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(GameFrame(window).IsVisible);
            Assert.Equal(new[] { "Actraiser", "Zelda" }, Titles(window));
            Assert.Contains("2 games", Header(window).Text);
        }, default);

        [Fact]
        public Task With_no_rom_directory_set_the_library_points_at_preferences() => Session.Dispatch(() =>
        {
            ConfigureRomDirectory(null);

            var window = new MainWindow();
            window.Show();

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(LibraryList(window).IsVisible);
            Assert.Contains("Preferences", Header(window).Text!);
        }, default);

        [Fact]
        public Task Selecting_a_title_switches_to_the_game_screen() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            Assert.True(LibraryView(window).IsVisible);

            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");

            Assert.False(LibraryView(window).IsVisible);
            Assert.True(GameFrame(window).IsVisible);
            Assert.Contains("Playable.smc", window.GetControl<TextBlock>("StatusText").Text!);

            window.Close();
        }, default);

        [Fact]
        public Task The_library_can_be_returned_to_after_a_game_starts() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Assert.True(GameFrame(window).IsVisible);

            Invoke(window, "ShowLibrary");

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(GameFrame(window).IsVisible);
            Assert.Equal("No ROM loaded", window.GetControl<TextBlock>("StatusText").Text);

            window.Close();
        }, default);

        [Fact]
        public Task A_rom_added_after_startup_appears_on_the_next_refresh() => Session.Dispatch(() =>
        {
            WriteRom("First.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            Assert.Single(Titles(window));

            WriteRom("Second.smc");
            Invoke(window, "RefreshLibrary");

            Assert.Equal(new[] { "First", "Second" }, Titles(window));
        }, default);

        // Flags being right does not prove the list laid out - see EmuSen_Settings_Reference.md §4.11.
        [Fact]
        public Task The_library_screen_actually_renders_its_titles() => Session.Dispatch(() =>
        {
            for (int i = 0; i < 6; i++) WriteRom($"Game{i}.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();

            byte[] pixels = EmuSen.WiseMan.Fixtures.UiTest.AssertLaidOut(window, "library").Rgba;

            // Every other pixel on this screen is black or grey text, so a
            // strongly colour-cast one can only be the selected row's accent -
            // i.e. a real templated ListBoxItem drew. Channel order is not
            // assumed; only that one of the outer channels dominates.
            bool selectionAccentDrawn = false;
            for (int i = 0; i + 3 < pixels.Length && !selectionAccentDrawn; i += 4)
            {
                int first = pixels[i];
                int third = pixels[i + 2];
                selectionAccentDrawn = (first > 120 && third < 80) || (third > 120 && first < 80);
            }
            Assert.True(selectionAccentDrawn, "No selected library row rendered - the list did not lay out.");
        }, default);

        // The search box is a FilterBar template part now, so this comes from the visual tree - see EmuSen_LunaP.md §14.2.
        private static TextBox SearchBox(MainWindow w) => w.FindNamed<TextBox>("PART_Search");

        // Real key events, not Text assignments: the bug was purely in routing - see EmuSen_Settings_Reference.md §4.17.
        private static void Press(MainWindow w, Key key, PhysicalKey physical) =>
            w.KeyPress(key, RawInputModifiers.None, physical, null);

        // Models X11Window.DispatchInput: no character follows a handled KeyDown - see EmuSen_Settings_Reference.md §4.17.
        private static void Type(MainWindow w, Key key, PhysicalKey physical, string character)
        {
            bool handled = false;
            void Probe(object? _, KeyEventArgs e) => handled = e.Handled;

            w.AddHandler(InputElement.KeyDownEvent, (EventHandler<KeyEventArgs>)Probe, RoutingStrategies.Bubble, handledEventsToo: true);
            w.KeyPress(key, RawInputModifiers.None, physical, character);
            w.RemoveHandler(InputElement.KeyDownEvent, (EventHandler<KeyEventArgs>)Probe);

            if (!handled) w.KeyTextInput(character);
        }

        [Fact]
        public Task The_search_box_accepts_a_character_that_is_also_a_hotkey() => Session.Dispatch(() =>
        {
            WriteRom("Pilotwings.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            SearchBox(window).Focus();

            Type(window, Key.P, PhysicalKey.P, "p");

            Assert.Equal("p", SearchBox(window).Text);
            Assert.Equal(new[] { "Pilotwings" }, Titles(window));
        }, default);

        [Fact]
        public Task Backspace_deletes_in_the_search_box_instead_of_rewinding() => Session.Dispatch(() =>
        {
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            var search = SearchBox(window);
            search.Text = "Zelda";
            search.CaretIndex = 5;
            search.Focus();

            Press(window, Key.Back, PhysicalKey.Backspace);

            Assert.Equal("Zeld", search.Text);
        }, default);

        [Fact]
        public Task A_focused_search_box_keeps_game_input_and_hotkeys_off_the_keyboard() => Session.Dispatch(() =>
        {
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            SearchBox(window).Focus();

            Type(window, Key.Z, PhysicalKey.Z, "z");
            Press(window, Key.Tab, PhysicalKey.Tab);

            Assert.False(KeyboardHeld(window)[(int)PadButton.B]);
            Assert.False(Field<bool>(window, "_turboHeld"));
            Assert.Equal("z", SearchBox(window).Text);
        }, default);

        // The guard above must stay scoped: a game on screen still takes every key.
        [Fact]
        public Task Game_input_and_hotkeys_still_work_while_a_game_is_on_screen() => Session.Dispatch(() =>
        {
            WriteRom("Anything.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Assert.True(GameFrame(window).IsVisible);

            Press(window, Key.Z, PhysicalKey.Z);
            Press(window, Key.Tab, PhysicalKey.Tab);

            Assert.True(KeyboardHeld(window)[(int)PadButton.B]);
            Assert.True(Field<bool>(window, "_turboHeld"));

            window.Close();
        }, default);

        [Fact]
        public Task Enter_in_the_search_box_starts_the_narrowed_selection() => Session.Dispatch(() =>
        {
            WriteRom("Actraiser.smc");
            WriteRom("Zelda.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            var search = SearchBox(window);
            search.Focus();

            Type(window, Key.Z, PhysicalKey.Z, "z");
            Assert.Equal(new[] { "Zelda" }, Titles(window));

            Press(window, Key.Enter, PhysicalKey.Enter);

            Assert.True(GameFrame(window).IsVisible);
            Assert.Contains("Zelda.smc", window.GetControl<TextBlock>("StatusText").Text!);

            // The search box it was launched from is hidden now, so the guard must not still be holding the keyboard.
            Press(window, Key.Z, PhysicalKey.Z);
            Assert.True(KeyboardHeld(window)[(int)PadButton.B]);

            window.Close();
        }, default);

        [Fact]
        public Task Escape_steps_out_of_a_game_and_pauses_it_without_unloading() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Assert.True(GameFrame(window).IsVisible);
            Assert.False(IsPaused(window));

            Press(window, Key.Escape, PhysicalKey.Escape);

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(GameFrame(window).IsVisible);
            Assert.True(IsPaused(window));
            // Suspended, not unloaded - ShowLibrary would have said "No ROM loaded".
            Assert.Equal("Paused: Playable.smc", window.GetControl<TextBlock>("StatusText").Text);
            Assert.Contains("Playable.smc", Hint(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task Escape_from_the_library_returns_to_the_suspended_game_running() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Press(window, Key.Escape, PhysicalKey.Escape);

            Press(window, Key.Escape, PhysicalKey.Escape);

            Assert.True(GameFrame(window).IsVisible);
            Assert.False(LibraryView(window).IsVisible);
            Assert.False(IsPaused(window));
            Assert.Equal("Running: Playable.smc", window.GetControl<TextBlock>("StatusText").Text);

            window.Close();
        }, default);

        [Fact]
        public Task Browsing_the_library_cannot_reach_a_suspended_game() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();
            LibraryList(window).SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
            Press(window, Key.Escape, PhysicalKey.Escape);
            LibraryList(window).Focus();

            // Navigating the list must not latch the pad, and P must not restart it off-screen.
            Press(window, Key.Down, PhysicalKey.ArrowDown);
            Press(window, Key.Z, PhysicalKey.Z);
            Press(window, Key.P, PhysicalKey.P);

            Assert.False(KeyboardHeld(window)[(int)PadButton.Down]);
            Assert.False(KeyboardHeld(window)[(int)PadButton.B]);
            Assert.True(IsPaused(window));

            window.Close();
        }, default);

        [Fact]
        public Task Escape_does_nothing_with_no_game_loaded() => Session.Dispatch(() =>
        {
            WriteRom("Playable.smc");
            ConfigureRomDirectory(_romDir);

            var window = new MainWindow();
            window.Show();

            Press(window, Key.Escape, PhysicalKey.Escape);

            Assert.True(LibraryView(window).IsVisible);
            Assert.False(GameFrame(window).IsVisible);
            Assert.Equal("No ROM loaded", window.GetControl<TextBlock>("StatusText").Text);
        }, default);

        private static TextBlock Hint(MainWindow w) => w.GetControl<TextBlock>("LibraryHintText");

        private static bool IsPaused(MainWindow window) =>
            (bool)typeof(MainWindow).GetProperty("IsPaused", BindingFlags.Instance | BindingFlags.NonPublic | BindingFlags.Public)!
                .GetValue(window)!;

        private static bool[] KeyboardHeld(MainWindow window) => Field<bool[]>(window, "_keyboardHeld");

        private static T Field<T>(MainWindow window, string name) =>
            (T)typeof(MainWindow)
                .GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!
                .GetValue(window)!;

        // These are private by design - the library screen is internal to
        // MainWindow, not API. Reflection here beats widening it for tests.
        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow)
                .GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!
                .Invoke(window, null);
    }
}
