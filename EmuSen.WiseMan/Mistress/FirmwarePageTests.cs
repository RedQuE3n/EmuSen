using System;
using System.Collections.Generic;
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

        private static HintText Problem(PreferencesWindow w) => w.GetVisualDescendants().OfType<HintText>().Single(t => t.Name == "FirmwareDirectoryProblem");

        private static PathPickerRow Picker(PreferencesWindow w) => w.GetVisualDescendants().OfType<PathPickerRow>().Single(p => p.Name == "FirmwareDirectoryBox");

        // A path typed into the folder row and committed by leaving the box, as a keyboard or Game Mode's on-screen keyboard does.
        private static void Type(PreferencesWindow w, string text)
        {
            TextBox box = Picker(w).GetVisualDescendants().OfType<TextBox>().Single();
            box.Focus();
            box.Text = text;
            Picker(w).GetVisualDescendants().OfType<Button>().Single(b => b.Name == "PART_Browse").Focus();
            Dispatcher.UIThread.RunJobs();
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
                // A file two systems use, as the Genesis and the Sega CD both use the TMSS ROM, has a row under each.
                List<FieldRow> rows = window.GetVisualDescendants().OfType<FieldRow>().Where(r => r.Label == item.Title).ToList();
                Assert.Equal(systems.Count(s => s.Items.Any(i => i.Title == item.Title)), rows.Count);
                Assert.All(rows, row =>
                {
                    Assert.Equal(new[] { item.InUse, item.Change, item.Where, item.Note }.Where(w => w is not null), Words(row));
                    Assert.Equal(item.Cost, (ToolTip.GetTip(row) as TextBlock)?.Text);
                });
            }
            FieldRow folder = RowOf(window, FirmwarePane.FolderRowLabel);
            Assert.Equal(FirmwareOverview.Optional, folder.Hint);
            Assert.Contains("never downloads", folder.Hint);
            Assert.DoesNotContain("below", folder.Hint);
            Assert.DoesNotContain("above", folder.Hint);
            Assert.Equal(FirmwareLibrary.Directory, window.GetVisualDescendants().OfType<MonoText>().Single(t => t.Name == "FirmwareFolderText").Text);
            FieldRow chosen = RowOf(window, FirmwarePane.ChosenLabel);
            Assert.Equal(FirmwarePane.ChosenHint, chosen.Hint);
            PathPickerRow picker = chosen.GetVisualDescendants().OfType<PathPickerRow>().Single();
            Assert.Equal(("", FirmwarePane.ChosenPlaceholder, true, true), (picker.Path, picker.Placeholder, picker.IsEditable, picker.IsClearable));
            Assert.False(Problem(window).IsVisible);
            window.Close();
        }, default);

        [Fact]
        public Task The_folder_s_button_opens_the_folder_and_is_the_page_s_only_button() => Session.Dispatch(() =>
        {
            PreferencesWindow window = Open();
            var pane = (Control)window.GetVisualDescendants().OfType<Tabs>().Single().SelectedContent!;
            // Besides the folder row's own Browse and Clear, Open Folder is the page's one button.
            Button open = Assert.Single(pane.GetVisualDescendants().OfType<Button>().Where(b => b.FindAncestorOfType<ScrollBar>() is null && b.FindAncestorOfType<PathPickerRow>() is null));
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
            Assert.Equal([FirmwareItem.OwnFile, FirmwareItem.AsTheConsole, "Found in EmuSen's folder."], Words(RowOf(window, title)));

            // One byte short is not the file: the open version runs and the row says why.
            File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, file), new byte[size - 1]);
            Assert.True(window.RefreshFirmware());
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(3, Words(RowOf(window, title)).Length);
            Assert.Equal($"Not used: the file in EmuSen's folder is not {size:N0} bytes.", Words(RowOf(window, title))[2]);
            window.Close();

            File.Delete(Path.Combine(FirmwareLibrary.Directory, file));
            window = Open();
            Assert.Equal(2, Words(RowOf(window, title)).Length);
            Assert.NotEqual(FirmwareItem.OwnFile, Words(RowOf(window, title))[0]);
            window.Close();
        }, default);

        // The folder row by the setting's rules: a relative path or a file refused and the row put back, a missing folder kept and said, ~ expanded, a real folder used, Clear back to blank.
        [Fact]
        public Task The_folder_row_keeps_a_folder_refuses_a_relative_path_or_a_file_and_clears() => Session.Dispatch(() =>
        {
            FirmwareItem first = FirmwareOverview.Build().SelectMany(s => s.Items).First();
            string chosen = Path.Combine(_root, "BIOS");
            Directory.CreateDirectory(chosen);
            File.WriteAllBytes(Path.Combine(chosen, first.FileName), new byte[first.Size]);
            PreferencesWindow window = Open();

            Type(window, "genesis");
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.Equal("", Picker(window).Path);
            Assert.True(Problem(window).IsVisible);
            Assert.Equal(SettingFolders.Checked("genesis").Problem, Problem(window).Text);

            string file = Path.Combine(chosen, first.FileName);
            Type(window, file);
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"\"{file}\" is a file, not a folder, so it is not used.", Problem(window).Text);

            string missing = Path.Combine(_root, "Unplugged");
            Type(window, missing);
            Assert.Equal(missing, AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"The folder {missing} does not exist.", Problem(window).Text);
            Assert.False(Directory.Exists(missing));

            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            Type(window, "~/EmuSenNoSuchFirmwareFolder");
            Assert.Equal("~/EmuSenNoSuchFirmwareFolder", AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"The folder {Path.Join(home, "EmuSenNoSuchFirmwareFolder")} does not exist.", Problem(window).Text);

            Type(window, chosen);
            Assert.Equal(chosen, AppSettings.Load().FirmwareDirectory);
            Assert.False(Problem(window).IsVisible);
            Assert.Equal([FirmwareItem.OwnFile, FirmwareItem.AsTheConsole, "Found in your folder."], Words(RowOf(window, first.Title)));

            Button clear = Picker(window).GetVisualDescendants().OfType<Button>().Single(b => b.Name == "PART_Clear");
            clear.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            Dispatcher.UIThread.RunJobs();
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.Equal("", Picker(window).Path);
            Assert.NotEqual(FirmwareItem.OwnFile, Words(RowOf(window, first.Title))[0]);
            Assert.Equal([first.FileName], Directory.GetFileSystemEntries(chosen).Select(Path.GetFileName));
            window.Close();
        }, default);
    }
}
