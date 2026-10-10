using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // A game's own cheat list: one folder per console, and kept when the game is left - see EmuSen_Config_Reference.md §3.4b.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameCheatListTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameCheatListTests).GetTypeInfo().Assembly);

        private readonly string _root;
        private readonly string _snes;
        private readonly string _nes;

        public GameCheatListTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenGameCheatTests", Guid.NewGuid().ToString("N"));
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings
            {
                RomDirectory = roms,
                LogDirectory = Path.Combine(_root, "Logs"),
                StateDirectory = Path.Combine(_root, "States"),
                ResumeOnLaunch = AppSettings.ResumeNever,
            }.Save();

            _snes = Path.Combine(roms, "Gamma.smc");
            _nes = Path.Combine(roms, "Gamma.nes");
            File.WriteAllBytes(_snes, SyntheticRom.BuildBlank());
            File.WriteAllBytes(_nes, SyntheticNesRom.Build());
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private static CheatRegistry Cheats(MainWindow window) =>
            (CheatRegistry)typeof(MainWindow).GetField("_cheats", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;

        private static void Play(MainWindow window, string path) =>
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { path, Path.GetFileName(path) });

        private static void Leave(MainWindow window) =>
            typeof(MainWindow).GetMethod("ShowLibrary", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);

        private static MainWindow Open()
        {
            var window = new MainWindow();
            window.Show();
            return window;
        }

        private string ListPath(string console) => Path.Combine(_root, "Config", CheatFile.CategoryDirName, console, "Gamma.json");

        [Fact]
        public Task Two_consoles_games_of_one_name_keep_separate_lists() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "snes only");
            Assert.True(GameCheats.For("SNES", _snes)!.Save(registry.ToCheatFile()));

            var window = Open();
            Play(window, _nes);
            Assert.Empty(Cheats(window).GetCheats());

            Play(window, _snes);
            Assert.Equal("snes only", Cheats(window).GetCheats().Single().Description);
            window.Close();
        }, default);

        [Fact]
        public Task A_list_changed_and_never_saved_is_kept_when_the_game_is_left() => Session.Dispatch(() =>
        {
            var first = Open();
            Play(first, _snes);
            Cheats(first).AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk", enabled: false);
            Leave(first);
            first.Close();

            Assert.True(File.Exists(ListPath("SNES")));

            var second = Open();
            Play(second, _snes);
            var kept = Cheats(second).GetCheats().Single();
            Assert.Equal("fast walk", kept.Description);
            Assert.False(kept.Enabled);
            second.Close();
        }, default);

        [Fact]
        public Task Closing_the_window_keeps_the_list_too() => Session.Dispatch(() =>
        {
            var window = Open();
            Play(window, _snes);
            Cheats(window).AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk");
            window.Close();

            Assert.Equal("fast walk", GameCheats.For("SNES", _snes)!.Load()!.Cheats.Single().Description);
        }, default);

        [Fact]
        public Task A_game_left_with_no_cheats_writes_nothing() => Session.Dispatch(() =>
        {
            var window = Open();
            Play(window, _snes);
            Leave(window);
            window.Close();

            Assert.False(Directory.Exists(Path.Combine(_root, "Config", CheatFile.CategoryDirName)));
        }, default);

        [Fact]
        public Task An_unchanged_list_is_not_written_again() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk");
            Assert.True(GameCheats.For("SNES", _snes)!.Save(registry.ToCheatFile()));
            var stamp = new DateTime(2001, 1, 1, 0, 0, 0, DateTimeKind.Utc);
            File.SetLastWriteTimeUtc(ListPath("SNES"), stamp);

            var window = Open();
            Play(window, _snes);
            Leave(window);
            window.Close();

            Assert.Equal(stamp, File.GetLastWriteTimeUtc(ListPath("SNES")));
        }, default);

        [Fact]
        public Task Removing_every_cheat_is_kept_so_they_do_not_come_back() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "fast walk");
            Assert.True(GameCheats.For("SNES", _snes)!.Save(registry.ToCheatFile()));

            var first = Open();
            Play(first, _snes);
            Cheats(first).Clear();
            Leave(first);
            first.Close();

            var second = Open();
            Play(second, _snes);
            Assert.Empty(Cheats(second).GetCheats());
            second.Close();
        }, default);

        [Fact]
        public Task A_flat_list_from_an_earlier_build_is_still_restored_and_left_untouched() => Session.Dispatch(() =>
        {
            var registry = new CheatRegistry();
            registry.AddRamPoke("CpuBus", 0x7E005E, 0x10, "from before");
            Assert.True(CheatFile.For("Gamma").Save(registry.ToCheatFile()));
            string flat = CheatFile.For("Gamma").Path;
            string before = File.ReadAllText(flat);

            var window = Open();
            Play(window, _snes);
            Assert.Equal("from before", Cheats(window).GetCheats().Single().Description);
            Cheats(window).AddRamPoke("CpuBus", 0x7E0060, 0x22, "added since");
            Leave(window);
            window.Close();

            Assert.Equal(before, File.ReadAllText(flat));
            Assert.Equal(2, GameCheats.For("SNES", _snes)!.Load()!.Cheats.Count);
        }, default);
    }
}
