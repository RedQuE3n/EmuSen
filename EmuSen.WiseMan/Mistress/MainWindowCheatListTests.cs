using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Interactivity;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Who owns the cheat list, and what a ROM change does to it - see
    // EmuSen_Settings_Reference.md §4.14.
    [Collection(TestCollections.ProcessGlobals)]
    public class MainWindowCheatListTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(MainWindowCheatListTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _romDir;

        public MainWindowCheatListTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenMainWindowCheatTests", Guid.NewGuid().ToString("N"));
            _romDir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(_romDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings
            {
                RomDirectory = _romDir,
                LogDirectory = Path.Combine(_root, "Logs"),
                StateDirectory = Path.Combine(_root, "States"),
            }.Save();

            // Two, so "a different ROM" is reachable. Sorted by title, so
            // Alpha is index 0 and Beta index 1.
            File.WriteAllBytes(Path.Combine(_romDir, "Alpha.smc"), SyntheticRom.BuildBlank());
            File.WriteAllBytes(Path.Combine(_romDir, "Beta.smc"), SyntheticRom.BuildBlank());
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static void Launch(MainWindow window, int index)
        {
            window.GetControl<ListBox>("LibraryList").SelectedIndex = index;
            Invoke(window, "LaunchSelectedLibraryEntry");
        }

        private static CheatRegistry Cheats(MainWindow window) => (CheatRegistry)Field(window, "_cheats")!;

        // Private by design - the cheat list is internal to MainWindow, not API.
        private static void Invoke(MainWindow window, string method) =>
            typeof(MainWindow).GetMethod(method, BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static object? Field(MainWindow window, string name) =>
            typeof(MainWindow).GetField(name, BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window);

        // The whole point of moving ownership out of SnesDebugTarget: the
        // core is rebuilt per load, the list is not.
        [Fact]
        public Task The_running_core_shares_the_windows_own_cheat_registry() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            Launch(window, 0);

            var target = (SnesDebugTarget?)Field(window, "_debugTarget");
            Assert.NotNull(target);
            Assert.Same(Cheats(window), target!.Cheats);

            window.Close();
        }, default);

        [Fact]
        public Task Cheats_can_be_loaded_before_any_rom_is() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();

            Cheats(window).AddRamPoke("WRAM", 0x9C, 0x63, "loaded first");
            Launch(window, 0);

            Assert.Equal("loaded first", Cheats(window).GetCheats()[0].Description);

            window.Close();
        }, default);

        // A reset reloads the same ROM, so the list it was built for is
        // still the right one.
        [Fact]
        public Task A_reset_keeps_the_cheat_list() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            Launch(window, 0);

            Cheats(window).AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");
            Invoke(window, "ResetEmulation");

            Assert.Equal("infinite lives", Cheats(window).GetCheats()[0].Description);

            window.Close();
        }, default);

        // Another game's addresses mean nothing here - keeping them would
        // poke this one's RAM every frame.
        [Fact]
        public Task A_different_rom_clears_the_cheat_list() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            Launch(window, 0);
            Cheats(window).AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");

            Launch(window, 1);

            Assert.Empty(Cheats(window).GetCheats());

            window.Close();
        }, default);

        // Closing a game nulls _currentRomPath, which must not be mistaken
        // for a ROM change when the same game is started again.
        [Fact]
        public Task Closing_and_reopening_the_same_rom_keeps_the_cheat_list() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            Launch(window, 0);
            Cheats(window).AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");

            Invoke(window, "ShowLibrary");
            Launch(window, 0);

            Assert.Equal("infinite lives", Cheats(window).GetCheats()[0].Description);

            window.Close();
        }, default);

        // The point of saving one: it comes back without being asked for.
        // See §4.15.
        [Fact]
        public Task A_saved_cheat_list_comes_back_when_that_rom_starts() => Session.Dispatch(() =>
        {
            var first = new MainWindow();
            first.Show();
            Launch(first, 0);
            Cheats(first).AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk");
            CheatFile.For("Alpha").Save(Cheats(first).ToCheatFile());
            first.Close();

            var second = new MainWindow();
            second.Show();
            Launch(second, 0);

            Assert.Equal("fast walk", Cheats(second).GetCheats().Single().Description);
            Assert.True(Cheats(second).GetCheats().Single().Enabled);

            second.Close();
        }, default);

        // Only that game's - Beta's list is not Alpha's.
        [Fact]
        public Task A_saved_list_is_not_restored_for_a_different_game() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk");
            CheatFile.For("Alpha").Save(registry.ToCheatFile());

            var window = new MainWindow();
            window.Show();
            Launch(window, 1); // Beta

            Assert.Empty(Cheats(window).GetCheats());

            window.Close();
        }, default);

        // A reset would otherwise throw away everything added since the save.
        [Fact]
        public Task Restoring_never_overwrites_a_list_already_in_hand() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "from disk");
            CheatFile.For("Alpha").Save(registry.ToCheatFile());

            var window = new MainWindow();
            window.Show();
            Launch(window, 0);
            Cheats(window).AddRamPoke("CpuBus", 0x7E0060, 0x22, "added since");

            Invoke(window, "ResetEmulation");
            Invoke(window, "ShowLibrary");
            Launch(window, 0);

            Assert.Contains(Cheats(window).GetCheats(), c => c.Description == "added since");

            window.Close();
        }, default);

        // The cheat database window edits the same list the Active Cheats
        // window shows - see §4.14.
        [Fact]
        public Task Both_cheat_windows_reach_the_windows_own_registry() => Session.Dispatch(() =>
        {
            var window = new MainWindow();
            window.Show();
            Cheats(window).AddRamPoke("WRAM", 0x9C, 0x63, "infinite lives");

            window.GetControl<MenuItem>("SettingsMenu").RaiseEvent(new RoutedEventArgs(MenuItem.SubmenuOpenedEvent));
            foreach (string header in new[] { "Chea_t Database...", "_Active Cheats..." })
            {
                MenuItem item = System.Linq.Enumerable.Single(
                    System.Linq.Enumerable.OfType<MenuItem>(window.GetControl<MenuItem>("SettingsMenu").Items),
                    m => (string?)m.Header == header);

                Assert.True(item.IsEnabled, $"{header} should not need a ROM.");
                item.RaiseEvent(new RoutedEventArgs(MenuItem.ClickEvent));
            }

            Assert.Single(Cheats(window).GetCheats());

            window.Close();
        }, default);
    }
}
