using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A session's logs land under the console that ran them - see EmuSen_Multicore.md §12.
    [Collection(TestCollections.ProcessGlobals)]
    public class MainWindowLogDirectoryTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowLogDirectoryTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;
        private readonly string _logDir;

        public MainWindowLogDirectoryTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenLogDirTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            _logDir = Path.Combine(_root, "Logs");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings { RomDirectory = _romDir, LogDirectory = _logDir }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, true); } catch { }
        }

        private static void Invoke(MainWindow w, string m) =>
            typeof(MainWindow).GetMethod(m, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(w, null);

        private string[] ConsoleFoldersUnderLogRoot() =>
            Directory.Exists(_logDir)
                ? Directory.GetDirectories(_logDir).Select(Path.GetFileName).OfType<string>().OrderBy(n => n).ToArray()
                : Array.Empty<string>();

        private void LaunchFirst(MainWindow window)
        {
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
            Invoke(window, "LaunchSelectedLibraryEntry");
        }

        [Fact]
        public Task An_snes_session_logs_under_SNES() => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.smc"), SyntheticRom.BuildBlank());

            var window = new MainWindow();
            window.Show();
            LaunchFirst(window);

            Assert.Equal(new[] { "SNES" }, ConsoleFoldersUnderLogRoot());
            window.Close();
        }, default);

        // The one that was wrong: every session logged under SNES, whatever ran.
        [Fact]
        public Task A_nes_session_logs_under_NES_not_SNES() => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Game.nes"), SyntheticNesRom.Build());

            var window = new MainWindow();
            window.Show();
            LaunchFirst(window);

            Assert.Equal(new[] { "NES" }, ConsoleFoldersUnderLogRoot());
            window.Close();
        }, default);

        [Fact]
        public Task Two_consoles_in_one_run_get_a_folder_each() => Session.Dispatch(() =>
        {
            File.WriteAllBytes(Path.Combine(_romDir, "Alpha.nes"), SyntheticNesRom.Build());
            File.WriteAllBytes(Path.Combine(_romDir, "Beta.smc"), SyntheticRom.BuildBlank());

            var window = new MainWindow();
            window.Show();
            LaunchFirst(window); // Alpha.nes, sorted first
            Invoke(window, "ShowLibrary");
            window.GetControl<ListBox>("LibraryList").SelectedIndex = 1;
            Invoke(window, "LaunchSelectedLibraryEntry");

            Assert.Equal(new[] { "NES", "SNES" }, ConsoleFoldersUnderLogRoot());
            window.Close();
        }, default);

        // The log path is resolved from the extension, so it has to agree with what the core reports.
        [Theory]
        [InlineData("game.smc", "SNES")]
        [InlineData("game.sfc", "SNES")]
        [InlineData("game.nes", "NES")]
        [InlineData("GAME.NES", "NES")]
        public void The_console_a_rom_resolves_to_matches_its_core(string fileName, string expected)
        {
            Assert.Equal(expected, CoreCatalog.ConsoleForRom(fileName));
        }

        [Fact]
        public void An_unhandled_extension_resolves_to_nothing()
        {
            Assert.Null(CoreCatalog.ConsoleForRom("notes.txt"));
        }
    }
}
