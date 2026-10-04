using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Quitting with Preferences open, as a window on the desktop or a sheet on the big screen, closes it without touching the closed records - see EmuSen_Settings_Reference.md §4.88.
    [Collection(TestCollections.ProcessGlobals)]
    public class MainWindowCloseTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowCloseTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenMainWindowClose", Guid.NewGuid().ToString("N"));

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public Task Closing_the_window_with_preferences_open_closes_preferences_cleanly(bool bigScreen) => Session.Dispatch(() =>
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            File.WriteAllBytes(Path.Combine(roms, "Game.sfc"), SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = roms, LibraryView = AppSettings.LibraryList, ResumeOnLaunch = AppSettings.ResumeNever, BigScreen = bigScreen }.Save();
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            typeof(MainWindow).GetMethod("ShowPreferences", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            Dispatcher.UIThread.RunJobs();
            PreferencesWindow preferences = window.GetControl<SheetLayer>("Sheets").Current as PreferencesWindow ?? window.OwnedWindows.OfType<PreferencesWindow>().Single();
            bool closed = false;
            preferences.Closed += (_, _) => closed = true;

            window.Close();

            Assert.True(closed, "Preferences is still open");
        }, default);
    }
}
