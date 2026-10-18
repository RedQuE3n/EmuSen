using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress;
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

        private static CheatList Cheats(MainWindow window) => MistressGame.Cheats(window);

        // A RAM poke as the session's list writes it to a game's file.
        private static CheatFileEntry Poke(uint address, uint value, string description, bool enabled = true) =>
            new() { Kind = "RamPoke", Description = description, Enabled = enabled, Writes = { new CheatFileWrite { Space = "CpuBus", Address = CheatFileWrite.Hex(address, 1), Value = CheatFileWrite.Hex(value, 2) } } };

        private static CheatFile ListOf(params CheatFileEntry[] cheats)
        {
            var file = new CheatFile();
            file.Cheats.AddRange(cheats);
            return file;
        }

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
            Assert.True(GameCheats.For("SNES", _snes)!.Save(ListOf(Poke(0x7E005E, 0x10, "snes only"))));

            var window = Open();
            Play(window, _nes);
            Assert.Empty(Cheats(window).Cheats);

            Play(window, _snes);
            Assert.Equal("snes only", Cheats(window).Cheats.Single().Description);
            window.Close();
        }, default);

        [Fact]
        public Task A_list_changed_and_never_saved_is_kept_when_the_game_is_left() => Session.Dispatch(() =>
        {
            var first = Open();
            Play(first, _snes);
            Cheats(first).Import(ListOf(Poke(0x7E005E, 0x10, "fast walk", enabled: false)));
            Leave(first);
            first.Close();

            Assert.True(File.Exists(ListPath("SNES")));

            var second = Open();
            Play(second, _snes);
            var kept = Cheats(second).Cheats.Single();
            Assert.Equal("fast walk", kept.Description);
            Assert.False(kept.Enabled);
            second.Close();
        }, default);

        [Fact]
        public Task Closing_the_window_keeps_the_list_too() => Session.Dispatch(() =>
        {
            var window = Open();
            Play(window, _snes);
            Cheats(window).Import(ListOf(Poke(0x7E005E, 0x10, "fast walk")));
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
            Assert.True(GameCheats.For("SNES", _snes)!.Save(ListOf(Poke(0x7E005E, 0x10, "fast walk"))));
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
            Assert.True(GameCheats.For("SNES", _snes)!.Save(ListOf(Poke(0x7E005E, 0x10, "fast walk"))));

            var first = Open();
            Play(first, _snes);
            Cheats(first).Clear();
            Leave(first);
            first.Close();

            var second = Open();
            Play(second, _snes);
            Assert.Empty(Cheats(second).Cheats);
            second.Close();
        }, default);

        [Fact]
        public Task A_flat_list_from_an_earlier_build_is_still_restored_and_left_untouched() => Session.Dispatch(() =>
        {
            Assert.True(CheatFile.For("Gamma").Save(ListOf(Poke(0x7E005E, 0x10, "from before"))));
            string flat = CheatFile.For("Gamma").Path;
            string before = File.ReadAllText(flat);

            var window = Open();
            Play(window, _snes);
            Assert.Equal("from before", Cheats(window).Cheats.Single().Description);
            Cheats(window).Import(ListOf(Poke(0x7E0060, 0x22, "added since")));
            Leave(window);
            window.Close();

            Assert.Equal(before, File.ReadAllText(flat));
            Assert.Equal(2, GameCheats.For("SNES", _snes)!.Load()!.Cheats.Count);
        }, default);
    }
}
