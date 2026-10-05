using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Headless;
using Avalonia.Interactivity;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Preferences' Firmware tab on the desktop: its rows are the overview's, the folder's button opens the folder and nothing else, and the rows follow the folder - see EmuSen_Settings_Reference.md §4.89.
    [Collection(TestCollections.ProcessGlobals)]
    public class FirmwarePageTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(FirmwarePageTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwarePage_" + Guid.NewGuid().ToString("N"));

        public FirmwarePageTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.ResetDirectory();
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
            NoSteam.Launcher.Clear();
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static PreferencesWindow Open()
        {
            var window = new PreferencesWindow(new AppSettings());
            window.Show();
            window.ShowTab(PreferencesWindow.FirmwareTab);
            Dispatcher.UIThread.RunJobs();
            return window;
        }

        private static FieldRow RowOf(PreferencesWindow w, string label) => w.GetVisualDescendants().OfType<FieldRow>().Single(r => r.Label == label);

        // A file's row: which version runs, then what the open one changes, then any note.
        private static string?[] Words(FieldRow row) => ((Panel)row.Content!).Children.OfType<TextBlock>().Select(t => t.Text).ToArray();

        [Fact]
        public Task The_tab_shows_every_system_and_each_file_s_words_from_the_overview() => Session.Dispatch(() =>
        {
            PreferencesWindow window = Open();
            var systems = FirmwareOverview.Build();
            Assert.Equal(systems.Select(s => s.Name), window.GetVisualDescendants().OfType<SectionHeader>().Select(h => h.Text));
            Assert.Equal(systems.Count(s => s.Items.Count == 0), window.GetVisualDescendants().OfType<HintText>().Count(h => h.Text == FirmwareOverview.NoFirmware));
            Assert.NotEmpty(systems.SelectMany(s => s.Items));
            foreach (FirmwareItem item in systems.SelectMany(s => s.Items))
            {
                FieldRow row = RowOf(window, item.Title);
                Assert.Equal(item.Note is { } note ? [item.InUse, item.Change, note] : [item.InUse, item.Change], Words(row));
                Assert.Equal(item.Cost, (ToolTip.GetTip(row) as TextBlock)?.Text);
            }
            FieldRow folder = RowOf(window, FirmwarePane.FolderLabel);
            Assert.Equal(FirmwareOverview.Optional, folder.Hint);
            Assert.Contains("never downloads", folder.Hint);
            Assert.DoesNotContain("below", folder.Hint);
            Assert.DoesNotContain("above", folder.Hint);
            Assert.Equal(FirmwareLibrary.Directory, window.GetVisualDescendants().OfType<MonoText>().Single(t => t.Name == "FirmwareFolderText").Text);
            window.Close();
        }, default);

        [Fact]
        public Task The_folder_s_button_opens_the_folder_and_is_the_page_s_only_button() => Session.Dispatch(() =>
        {
            PreferencesWindow window = Open();
            var pane = (Control)window.GetVisualDescendants().OfType<Tabs>().Single().SelectedContent!;
            Button open = Assert.Single(pane.GetVisualDescendants().OfType<Button>().Where(b => b.FindAncestorOfType<ScrollBar>() is null));
            Assert.Equal((FirmwarePane.OpenFolder, "FirmwareOpenFolderButton"), (open.Content, open.Name));
            Assert.False(Directory.Exists(FirmwareLibrary.Directory));
            open.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Assert.Equal([FirmwareLibrary.Directory], NoSteam.Launcher.Opened);
            Assert.True(Directory.Exists(FirmwareLibrary.Directory));
            Assert.Empty(Directory.GetFileSystemEntries(FirmwareLibrary.Directory));
            Assert.Empty(window.OwnedWindows);
            window.Close();
        }, default);

        [Fact]
        public Task The_rows_follow_the_folder_when_the_window_comes_back_and_when_it_is_opened_again() => Session.Dispatch(() =>
        {
            string title = FirmwareOverview.Build().SelectMany(s => s.Items).First().Title;
            string file = FirmwareOverview.Build().SelectMany(s => s.Items).First().FileName;
            int size = FirmwareOverview.Build().SelectMany(s => s.Items).First().Size;
            PreferencesWindow window = Open();
            Assert.NotEqual(FirmwareItem.OwnFile, Words(RowOf(window, title))[0]);
            Assert.False(window.RefreshFirmware());

            Directory.CreateDirectory(FirmwareLibrary.Directory);
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, file), new byte[size]);
            Assert.True(window.RefreshFirmware());
            Dispatcher.UIThread.RunJobs();
            Assert.Equal([FirmwareItem.OwnFile, FirmwareItem.AsTheConsole], Words(RowOf(window, title)));

            // One byte short is not the file: the open version runs and the row says why.
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, file), new byte[size - 1]);
            Assert.True(window.RefreshFirmware());
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(3, Words(RowOf(window, title)).Length);
            Assert.Equal($"The file here is not {size:N0} bytes, so it is not used.", Words(RowOf(window, title))[2]);
            window.Close();

            File.Delete(Path.Combine(FirmwareLibrary.Directory, file));
            window = Open();
            Assert.Equal(2, Words(RowOf(window, title)).Length);
            Assert.NotEqual(FirmwareItem.OwnFile, Words(RowOf(window, title))[0]);
            window.Close();
        }, default);
    }
}
