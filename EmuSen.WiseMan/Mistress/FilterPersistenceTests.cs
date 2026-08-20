using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Filters that outlive a session - see EmuSen_Settings_Reference.md §4.23.
    [Collection(TestCollections.ProcessGlobals)]
    public class FilterPersistenceTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FilterPersistenceTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;
        private readonly string _dbDir;

        public FilterPersistenceTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenFilterPersistence", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            _dbDir = Path.Combine(_root, "Cheats");
            Directory.CreateDirectory(_romDir);
            Directory.CreateDirectory(_dbDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void WriteRom(string fileName) =>
            File.WriteAllBytes(Path.Combine(_romDir, fileName), SyntheticRom.BuildBlank());

        [Fact]
        public Task The_library_search_is_restored_into_the_filter_bar() => Session.Dispatch(() =>
        {
            WriteRom("Super Mario World.sfc");
            WriteRom("Chrono Trigger.sfc");
            new AppSettings { RomDirectory = _romDir, LibrarySearch = "chrono" }.Save();

            var window = new MainWindow();
            window.Show();

            Assert.Equal("chrono", window.GetControl<FilterBar>("LibraryFilter").SearchText);

            window.Close();
        }, default);

        [Fact]
        public Task Closing_the_main_window_keeps_what_was_typed() => Session.Dispatch(() =>
        {
            WriteRom("Super Mario World.sfc");
            new AppSettings { RomDirectory = _romDir }.Save();

            var window = new MainWindow();
            window.Show();
            window.GetControl<FilterBar>("LibraryFilter").SearchText = "mario";
            window.Close();

            Assert.Equal("mario", AppSettings.Load().LibrarySearch);
        }, default);

        [Fact]
        public Task The_cheat_search_is_restored_and_kept() => Session.Dispatch(() =>
        {
            new AppSettings { CheatDatabaseDirectory = _dbDir, CheatSearch = "zelda" }.Save();

            var window = new CheatDatabaseWindow(AppSettings.Load());
            window.Show();

            Assert.Equal("zelda", window.GetControl<FilterBar>("GameFilter").SearchText);

            window.GetControl<FilterBar>("GameFilter").SearchText = "metroid";
            window.Close();

            Assert.Equal("metroid", AppSettings.Load().CheatSearch);
        }, default);

        // The LunaP guarantee this feature rests on - see EmuSen_Settings_Reference.md §4.23.
        [Fact]
        public Task Restoring_a_search_does_not_raise_Changed() => Session.Dispatch(() =>
        {
            var bar = new FilterBar();
            var host = new Window { Content = bar };
            host.Show();

            int changed = 0;
            bar.Changed += () => changed++;
            bar.SearchText = "restored";

            Assert.Equal(0, changed);
            Assert.Equal("restored", bar.SearchText);

            host.Close();
        }, default);
    }
}
