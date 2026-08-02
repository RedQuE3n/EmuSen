using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia;
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

            var frame = window.CaptureRenderedFrame()!;
            int width = frame.PixelSize.Width;
            int height = frame.PixelSize.Height;
            var pixels = new byte[width * height * 4];
            using (var fb = frame.Lock()) System.Runtime.InteropServices.Marshal.Copy(fb.Address, pixels, 0, pixels.Length);

            string? dump = Environment.GetEnvironmentVariable("EMUSEN_UI_DUMP");
            if (!string.IsNullOrEmpty(dump))
            {
                Directory.CreateDirectory(Path.GetDirectoryName(dump)!);
                EmuSen.Hotaru.Imaging.FrameImageWriter.SavePng(pixels, width, height, dump);
            }

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

        // These are private by design - the library screen is internal to
        // MainWindow, not API. Reflection here beats widening it for tests.
        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow)
                .GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!
                .Invoke(window, null);
    }
}
