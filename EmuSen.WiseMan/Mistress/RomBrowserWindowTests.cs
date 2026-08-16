using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.Common;
using EmuSen.Galaxia;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The quick alternative to the OS file picker - see EmuSen_Settings_Reference.md §4.11a.
    // Written when the window was migrated onto LunaList and found to have had no
    // coverage at all: nothing pinned that picking a row returns that row's path.
    [Collection(TestCollections.ProcessGlobals)]
    public class RomBrowserWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(RomBrowserWindowTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;

        public RomBrowserWindowTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenRomBrowserTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void WriteRom(string name) =>
            File.WriteAllBytes(Path.Combine(_romDir, name), SyntheticRom.BuildBlank());

        private static ListBox List(RomBrowserWindow w) => w.GetControl<ListBox>("RomList");

        private static void ClickOpen(RomBrowserWindow w) =>
            w.GetControl<Button>("OpenButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));

        [Fact]
        public Task It_lists_the_roms_in_the_folder() => Session.Dispatch(() =>
        {
            WriteRom("Zelda.smc");
            WriteRom("Actraiser.sfc");

            var window = new RomBrowserWindow(_romDir);
            window.Show();

            Assert.Equal(
                new[] { "Actraiser.sfc", "Zelda.smc" },
                List(window).ItemsSource!.Cast<string>().ToArray());

            window.Close();
        }, default);

        // The whole point of the window: the row picked is the path returned.
        [Fact]
        public Task Opening_a_selected_row_returns_that_rows_full_path() => Session.Dispatch(() =>
        {
            WriteRom("Zelda.smc");
            WriteRom("Actraiser.sfc");

            var window = new RomBrowserWindow(_romDir);
            var owner = new Window();
            owner.Show();
            Task<string?> result = window.ShowDialog<string?>(owner);

            List(window).SelectedIndex = 1;
            ClickOpen(window);

            Assert.Equal(Path.Combine(_romDir, "Zelda.smc"), result.Result);
        }, default);

        // Selecting is not choosing - LunaList.Chose fires on a selection change,
        // so wiring the dialog's close to it would end it on one click.
        [Fact]
        public Task Selecting_a_row_does_not_close_the_dialog() => Session.Dispatch(() =>
        {
            WriteRom("Zelda.smc");

            var window = new RomBrowserWindow(_romDir);
            var owner = new Window();
            owner.Show();
            Task<string?> result = window.ShowDialog<string?>(owner);

            List(window).SelectedIndex = 0;

            Assert.False(result.IsCompleted, "Choosing a row must take a double-click or the Open button.");

            window.Close(null);
        }, default);

        [Fact]
        public Task Opening_with_nothing_selected_does_nothing() => Session.Dispatch(() =>
        {
            WriteRom("Zelda.smc");

            var window = new RomBrowserWindow(_romDir);
            var owner = new Window();
            owner.Show();
            Task<string?> result = window.ShowDialog<string?>(owner);

            ClickOpen(window);

            Assert.False(result.IsCompleted);

            window.Close(null);
        }, default);

        [Fact]
        public Task An_empty_folder_says_so_instead_of_listing_nothing() => Session.Dispatch(() =>
        {
            var window = new RomBrowserWindow(_romDir);
            window.Show();

            Assert.False(string.IsNullOrWhiteSpace(window.GetControl<EmuSen.LunaP.Controls.HintText>("DirectoryText").Text));

            window.Close();
        }, default);
    }
}
