using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress
{
    // The pad read where a real one is read, so SDL's buttons, the stick threshold and the release wait are under test - see EmuSen_Settings_Reference.md §4.45.
    [Collection(TestCollections.ProcessGlobals)]
    public class PadInputPathTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(PadInputPathTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenPadInputPathTests", Guid.NewGuid().ToString("N"));
        private readonly string _romDir;

        public PadInputPathTests()
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

        private (MainWindow, PadDriver) LibraryOf(int games)
        {
            for (int i = 0; i < games; i++)
                File.WriteAllBytes(Path.Combine(_romDir, $"Game {i:00}.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = _romDir, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever }.Save();

            var window = new MainWindow();
            window.Show();
            return (window, new PadDriver(window));
        }

        private static void Stop(MainWindow window) =>
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        [Fact]
        public Task The_stick_moves_the_library_past_its_threshold_and_not_short_of_it() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = LibraryOf(4);
            var list = window.GetControl<ListBox>("LibraryList");
            list.SelectedIndex = 0;

            pad.Push(SDL.GamepadAxis.LeftY, 0.5);
            Assert.Equal(0, list.SelectedIndex);

            pad.Push(SDL.GamepadAxis.LeftY, 0.6);
            Assert.Equal(1, list.SelectedIndex);

            pad.Down(2);
            Assert.Equal(3, list.SelectedIndex);
            window.Close();
        }, default);

        [Fact]
        public Task South_starts_a_game_Start_alone_stays_the_game_s_and_the_chord_opens_the_menu() => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = LibraryOf(2);
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;

            pad.A();
            Assert.True(window.GetControl<Control>("GameFrame").IsVisible);

            // Start alone is the game's button, not the interface's - §4.29.
            pad.Start();
            Assert.False(window.GetControl<Control>("PadMenuPanel").IsVisible);

            pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);

            pad.B();
            Assert.False(window.GetControl<Control>("PadMenuPanel").IsVisible);

            pad.Guide();
            Assert.True(window.GetControl<Control>("PadMenuPanel").IsVisible);
            pad.B();

            Stop(window);
            window.Close();
        }, default);
    }
}
