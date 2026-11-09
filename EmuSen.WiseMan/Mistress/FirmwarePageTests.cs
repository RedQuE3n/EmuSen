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
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A console's Firmware page in Settings on the desktop: a row per need naming its files, the folder rows by the setting's rules, EmuSen's folder opened, and the rows following the folders - see EmuSen_BigPicture.md §52 and EmuSen_Settings_Reference.md §4.89.
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
            NoSteam.Launcher.Clear();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The Super Nintendo's firmware page, whose chips the runtime's overview names.
        private static MainWindow Open() => SettingsDesk.Open(SettingsScreen.Consoles, "console:SNES", "firmware");

        private sealed record FirmwareItem(string FileName, int Size);

        private static IReadOnlyList<FirmwareItem> Snes
        {
            get
            {
                using var runtime = new EmuSen.Crystal.Client.CrystalRuntime();
                using var overview = System.Text.Json.JsonDocument.Parse(runtime.FirmwareOverview());
                string name = EmuSen.Mistress.Library.RuntimeCatalog.Shelves.Single(s => s.EsdeSystem == "snes").EsdeFullName;
                return overview.RootElement.GetProperty("systems").EnumerateArray().Single(s => s.GetProperty("name").GetString() == name).GetProperty("items").EnumerateArray()
                    .Select(i => new FirmwareItem(i.GetProperty("file").GetString()!, i.GetProperty("size").GetInt32())).ToList();
            }
        }

        // The need's row that names the file.
        private static string RowFor(MainWindow window, string file) =>
            SettingsDesk.Layout(window).Rows.Single(r => r.Key.StartsWith("fw:", StringComparison.Ordinal) && (r.Sentence ?? "").Split(", ").Contains(file)).Key;

        private static string Pane(MainWindow window, string row) => string.Join(" ", window.SettingsShown!.PaneOf(row)!.Lines.Select(l => l.Text));

        private static string Status(MainWindow window) => window.GetControl<TextBlock>("StatusText").Text ?? "";

        private static void Folder(MainWindow window, string typed)
        {
            SettingsDesk.Choose(window, "fwfolder");
            SettingsDesk.Answer(window, typed);
        }

        [Fact]
        public Task The_page_has_a_row_for_every_file_the_overview_names_and_the_folder_rows() => Session.Dispatch(() =>
        {
            MainWindow window = Open();
            Assert.NotEmpty(Snes);
            foreach (FirmwareItem item in Snes) Assert.StartsWith("fw:", RowFor(window, item.FileName));
            Assert.Equal("EmuSen's folder", SettingsDesk.Value(window, "fwfolder"));
            Assert.Equal(["fwreplacement", "fwfolder", "fwclear", "fwopen", "fwcheck"], SettingsDesk.Keys(window).Where(k => !k.StartsWith("fw:", StringComparison.Ordinal)));
            Assert.Contains("never downloads", string.Join(" ", window.SettingsShown!.PaneOf(RowFor(window, Snes[0].FileName))!.Lines.Select(l => l.Text)));
            window.Close();
        }, default);

        [Fact]
        public Task Open_makes_and_opens_emusens_folder_and_nothing_else() => Session.Dispatch(() =>
        {
            MainWindow window = Open();
            Assert.False(Directory.Exists(DataStore.Firmware));
            SettingsDesk.Choose(window, "fwopen");
            Assert.Equal([DataStore.Firmware], NoSteam.Launcher.Opened);
            Assert.True(Directory.Exists(DataStore.Firmware));
            Assert.Empty(Directory.GetFileSystemEntries(DataStore.Firmware));
            Assert.Single(window.OwnedWindows);
            window.Close();
        }, default);

        [Fact]
        public Task The_rows_follow_the_folder_on_check_again_and_when_the_page_is_opened_again() => Session.Dispatch(() =>
        {
            FirmwareItem first = Snes[0];
            MainWindow window = Open();
            string row = RowFor(window, first.FileName);
            Assert.DoesNotContain("Found", SettingsDesk.Value(window, row));
            Assert.Contains("The open version: Games may start a fraction of a second later.", Pane(window, row));

            Directory.CreateDirectory(DataStore.Firmware);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, first.FileName), new byte[first.Size]);
            SettingsDesk.Choose(window, "fwcheck");
            Assert.StartsWith("✓ Found", SettingsDesk.Value(window, row));
            Assert.Contains($"{first.FileName}: found in EmuSen's folder.", Pane(window, row));

            // One byte short is not the file: the open version runs.
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, first.FileName), new byte[first.Size - 1]);
            SettingsDesk.Choose(window, "fwcheck");
            Assert.DoesNotContain("Found", SettingsDesk.Value(window, row));
            Assert.Contains($"{first.FileName}: not used, the file in EmuSen's folder is not {first.Size:N0} bytes.", Pane(window, row));

            File.Delete(Path.Combine(DataStore.Firmware, first.FileName));
            SettingsDesk.At(window, SettingsScreen.Consoles, "console:SNES", "firmware");
            Assert.DoesNotContain("Found", SettingsDesk.Value(window, row));
            window.Close();
        }, default);

        // The folder row by the setting's rules: a relative path or a file refused and said, a missing folder kept and said, ~ kept as typed, a real folder used, blank back to EmuSen's.
        [Fact]
        public Task The_folder_row_keeps_a_folder_refuses_a_relative_path_or_a_file_and_clears() => Session.Dispatch(() =>
        {
            FirmwareItem first = Snes[0];
            string chosen = Path.Combine(_root, "BIOS");
            Directory.CreateDirectory(chosen);
            File.WriteAllBytes(Path.Combine(chosen, first.FileName), new byte[first.Size]);
            MainWindow window = Open();

            Folder(window, "genesis");
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.Equal(SettingFolders.Checked("genesis").Problem, Status(window));

            string file = Path.Combine(chosen, first.FileName);
            Folder(window, file);
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"\"{file}\" is a file, not a folder, so it is not used.", Status(window));

            string missing = Path.Combine(_root, "Unplugged");
            Folder(window, missing);
            Assert.Equal(missing, AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"The folder {missing} does not exist.", Status(window));
            Assert.False(Directory.Exists(missing));

            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            Folder(window, "~/EmuSenNoSuchFirmwareFolder");
            Assert.Equal("~/EmuSenNoSuchFirmwareFolder", AppSettings.Load().FirmwareDirectory);
            Assert.Equal($"The folder {Path.Join(home, "EmuSenNoSuchFirmwareFolder")} does not exist.", Status(window));

            Folder(window, chosen);
            Assert.Equal(chosen, AppSettings.Load().FirmwareDirectory);
            Assert.Equal("", Status(window));
            Assert.Equal(chosen, SettingsDesk.Value(window, "fwfolder"));
            Assert.StartsWith("✓ Found", SettingsDesk.Value(window, RowFor(window, first.FileName)));
            Assert.Contains($"{first.FileName}: found in your folder.", Pane(window, RowFor(window, first.FileName)));

            SettingsDesk.Choose(window, "fwclear");
            Assert.Null(AppSettings.Load().FirmwareDirectory);
            Assert.False(SettingsDesk.Row(window, "fwclear").IsEnabled);
            Assert.Equal("EmuSen's folder", SettingsDesk.Value(window, "fwfolder"));
            Assert.DoesNotContain("Found", SettingsDesk.Value(window, RowFor(window, first.FileName)));
            Assert.Equal([first.FileName], Directory.GetFileSystemEntries(chosen).Select(Path.GetFileName));
            window.Close();
        }, default);
    }
}
