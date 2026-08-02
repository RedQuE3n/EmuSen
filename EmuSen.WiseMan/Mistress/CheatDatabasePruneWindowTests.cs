using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // The Prune button's two-stage confirm - see EmuSen_Settings_Reference.md §4.16.
    public class CheatDatabasePruneWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CheatDatabasePruneWindowTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _dbDir;

        private static readonly string[] Snes = { "Nintendo - Super Nintendo Entertainment System" };

        public CheatDatabasePruneWindowTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenPruneWindow", Guid.NewGuid().ToString("N"));
            _dbDir = Path.Combine(_root, "Cheats");
            Directory.CreateDirectory(_dbDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void WriteSystem(string system, int files)
        {
            string dir = Path.Combine(_dbDir, system);
            Directory.CreateDirectory(dir);
            for (int i = 0; i < files; i++) File.WriteAllText(Path.Combine(dir, $"Game {i}.cht"), "cheats = 0\n");
        }

        private AppSettings Settings() => new() { CheatDatabaseDirectory = _dbDir };

        private CheatDatabaseWindow Open(Func<IReadOnlyCollection<string>>? supported)
        {
            var window = new CheatDatabaseWindow(Settings(), null, null, null, null, supported);
            window.Show();
            return window;
        }

        private static void Click(CheatDatabaseWindow w) =>
            w.GetControl<Button>("PruneButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        private static string Status(CheatDatabaseWindow w) => w.GetControl<TextBlock>("StatusText").Text!;
        private static string Label(CheatDatabaseWindow w) => (string)w.GetControl<Button>("PruneButton").Content!;

        // Deleting is irreversible, so one click must never be enough.
        [Fact]
        public Task The_first_click_reports_and_deletes_nothing() => Session.Dispatch(() =>
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            WriteSystem("Sony - PlayStation", 3);
            var window = Open(() => Snes);

            Click(window);

            Assert.Contains("no core in this build", Status(window));
            Assert.Contains("Delete 1 system(s)?", Label(window));
            Assert.True(Directory.Exists(Path.Combine(_dbDir, "Sony - PlayStation")));

            window.Close();
        }, default);

        [Fact]
        public Task The_second_click_deletes_and_the_list_updates() => Session.Dispatch(() =>
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            WriteSystem("Sony - PlayStation", 3);
            var window = Open(() => Snes);

            Click(window);
            Click(window);

            Assert.Contains("Deleted 1 system(s), 3 file(s)", Status(window));
            Assert.False(Directory.Exists(Path.Combine(_dbDir, "Sony - PlayStation")));
            Assert.Equal("Prune Unsupported", Label(window));

            // The systems pane must not still be offering what was deleted.
            Assert.Single(window.GetControl<ListBox>("SystemsList").ItemsSource!);

            window.Close();
        }, default);

        [Fact]
        public Task Nothing_to_prune_says_so_and_stays_disarmed() => Session.Dispatch(() =>
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            var window = Open(() => Snes);

            Click(window);

            Assert.Contains("Nothing to prune", Status(window));
            Assert.Equal("Prune Unsupported", Label(window));

            window.Close();
        }, default);

        // A keep-set matching nothing is a wrong mapping, not an instruction
        // to empty the database.
        [Fact]
        public Task A_keep_set_matching_nothing_refuses_from_the_button_too() => Session.Dispatch(() =>
        {
            WriteSystem("Sony - PlayStation", 3);
            var window = Open(() => Snes);

            Click(window);
            Click(window);

            Assert.Contains("refusing to delete the whole database", Status(window));
            Assert.True(Directory.Exists(Path.Combine(_dbDir, "Sony - PlayStation")));

            window.Close();
        }, default);

        [Fact]
        public Task The_button_is_off_when_nobody_says_what_is_supported() => Session.Dispatch(() =>
        {
            WriteSystem("Sony - PlayStation", 3);
            var window = Open(null);

            Assert.False(window.GetControl<Button>("PruneButton").IsEnabled);

            window.Close();
        }, default);
    }
}
