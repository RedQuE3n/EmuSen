using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Reflection;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // The GUI half of the cheat database - see `man cheat`.
    public class CheatDatabaseWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CheatDatabaseWindowTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _dbDir;

        public CheatDatabaseWindowTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenCheatDbWindowTests", Guid.NewGuid().ToString("N"));
            _dbDir = Path.Combine(_root, "Cheats");
            Directory.CreateDirectory(_dbDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            CheatDatabaseInstaller.FetchOverride = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private const string OneCheat = """
            cheats = 1
            cheat0_desc = "Infinite Maximum Coins"
            cheat0_code = "7E0DBF63"
            cheat0_enable = false
            """;

        private void WriteChtFile(string system, string game)
        {
            string dir = Path.Combine(_dbDir, system);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, game + ".cht"), OneCheat);
        }

        private AppSettings Settings() => new() { CheatDatabaseDirectory = _dbDir };

        private static Stream BuildZip(params string[] paths)
        {
            var buffer = new MemoryStream();
            using (var archive = new ZipArchive(buffer, ZipArchiveMode.Create, leaveOpen: true))
            {
                foreach (string path in paths)
                {
                    using var writer = new StreamWriter(archive.CreateEntry(path).Open(), Encoding.UTF8);
                    writer.Write(OneCheat);
                }
            }
            buffer.Position = 0;
            return buffer;
        }

        private static TextBlock Status(CheatDatabaseWindow w) => w.GetControl<TextBlock>("StatusText");

        [Fact]
        public Task An_existing_folder_is_listed_by_system_on_open() => Session.Dispatch(() =>
        {
            WriteChtFile("Nintendo - Super Nintendo Entertainment System", "Super Mario World (USA)");
            WriteChtFile("Nintendo - Super Nintendo Entertainment System", "Super Metroid (Japan, USA)");
            WriteChtFile("Sony - PlayStation", "Final Fantasy VII (USA) (Disc 1)");

            var window = new CheatDatabaseWindow(Settings());
            window.Show();

            var systems = window.GetControl<ListBox>("SystemsList").ItemsSource!.Cast<string>().ToList();
            Assert.Equal(2, systems.Count);
            Assert.Contains(systems, s => s.Contains("Nintendo") && s.Contains("(2)"));
            Assert.Contains("3 cheat file(s)", Status(window).Text!);

            window.Close();
        }, default);

        [Fact]
        public Task An_empty_folder_says_so_rather_than_looking_broken() => Session.Dispatch(() =>
        {
            var window = new CheatDatabaseWindow(Settings());
            window.Show();

            Assert.Contains("No cheat files", Status(window).Text!);
            Assert.Empty(window.GetControl<ListBox>("SystemsList").ItemsSource!.Cast<string>());

            window.Close();
        }, default);

        // CC BY-SA 4.0 requires it, and it has to be visible before the
        // download, not after.
        [Fact]
        public Task The_attribution_is_on_screen_before_anything_is_downloaded() => Session.Dispatch(() =>
        {
            var window = new CheatDatabaseWindow(Settings());
            window.Show();

            string attribution = window.GetControl<TextBlock>("AttributionText").Text!;
            Assert.Contains("CC BY-SA 4.0", attribution);
            Assert.Contains("libretro", attribution);
            Assert.Contains("GameHacking.org", attribution);
            Assert.Contains("EmuSen ships none of this data", attribution);

            window.Close();
        }, default);

        [Fact]
        public Task Downloading_installs_and_relists_without_a_reopen() => Session.Dispatch(async () =>
        {
            CheatDatabaseInstaller.FetchOverride = _ => BuildZip(
                "cht/Nintendo - Super Nintendo Entertainment System/Super Mario World (USA).cht",
                "cht/Sony - PlayStation/Final Fantasy VII (USA) (Disc 1).cht");

            var window = new CheatDatabaseWindow(Settings());
            window.Show();
            Assert.Contains("No cheat files", Status(window).Text!);

            window.GetControl<Button>("DownloadButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            await WaitFor(() => Status(window).Text!.Contains("Installed"));

            Assert.Contains("Installed 2 cheat file(s)", Status(window).Text!);
            Assert.Equal(2, window.GetControl<ListBox>("SystemsList").ItemsSource!.Cast<string>().Count());

            window.Close();
        }, default);

        [Fact]
        public Task A_failed_download_reports_it_and_re_enables_the_button() => Session.Dispatch(async () =>
        {
            CheatDatabaseInstaller.FetchOverride = _ => throw new IOException("network is down");

            var window = new CheatDatabaseWindow(Settings());
            window.Show();

            window.GetControl<Button>("DownloadButton").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
            await WaitFor(() => Status(window).Text!.Contains("failed"));

            Assert.Contains("network is down", Status(window).Text!);
            Assert.True(window.GetControl<Button>("DownloadButton").IsEnabled, "the button must come back after a failure");
            Assert.True(window.GetControl<Button>("BrowseButton").IsEnabled);

            window.Close();
        }, default);

        [Fact]
        public Task With_no_folder_configured_it_falls_back_to_the_sandbox_default() => Session.Dispatch(() =>
        {
            var window = new CheatDatabaseWindow(new AppSettings { CheatDatabaseDirectory = null });
            window.Show();

            Assert.Contains("Cheats", Status(window).Text!);
            Assert.True(string.IsNullOrEmpty(window.GetControl<TextBox>("DirectoryBox").Text));

            window.Close();
        }, default);

        [Fact]
        public Task The_settings_menu_opens_it_without_needing_a_rom() => Session.Dispatch(() =>
        {
            new AppSettings { CheatDatabaseDirectory = _dbDir }.Save();

            var main = new MainWindow();
            main.Show();

            main.GetControl<MenuItem>("SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));

            MenuItem item = main.GetControl<MenuItem>("SettingsMenu").Items
                .OfType<MenuItem>()
                .Single(m => (string?)m.Header == "Chea_t Database...");

            Assert.True(item.IsEnabled);
            item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));

            main.Close();
        }, default);

        // The install runs off the UI thread, so the assertions have to wait
        // for it rather than assuming it finished.
        private static async Task WaitFor(Func<bool> condition)
        {
            for (int i = 0; i < 200 && !condition(); i++) await Task.Delay(10);
        }
    }
}
