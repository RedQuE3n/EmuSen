using System;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `cheat save`/`load`/`files` against a real target - see
    // EmuSen_Config_Reference.md §3.4 and `man cheat`.
    public class CheatPersistenceCommandTests : IDisposable
    {
        private readonly string _dir;
        private readonly SnesDebugTarget _target = BuildTarget();
        private readonly CheatCommand _command = new();

        public CheatPersistenceCommandTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenCheatCmd_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static SnesDebugTarget BuildTarget()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            return new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
        }

        private string Run(params string[] args) => _command.Execute(_target, args, null).Output;

        [Fact]
        public void Save_then_clear_then_load_restores_the_cheat()
        {
            _target.Cheats.AddRamPoke("WRAM", 0x0019, 0x09, "99 lives");

            Assert.Contains("Saved 1 cheat", Run("cheat", "save", "zelda"));

            _target.Cheats.Clear();
            Assert.Empty(_target.Cheats.GetCheats());

            Assert.Contains("Loaded 1 cheat", Run("cheat", "load", "zelda"));
            CheatInfo restored = _target.Cheats.GetCheats().Single();
            Assert.Equal(0x0019, restored.Address);
            Assert.Equal("99 lives", restored.Description);
        }

        // Documented behaviour, not an accident - `cheat clear` first to replace.
        [Fact]
        public void Load_adds_to_what_is_already_there_rather_than_replacing_it()
        {
            _target.Cheats.AddRamPoke("WRAM", 0x0019, 0x09, "saved");
            Run("cheat", "save", "zelda");

            Run("cheat", "load", "zelda");

            Assert.Equal(2, _target.Cheats.GetCheats().Count);
        }

        [Fact]
        public void Loading_a_file_that_is_not_there_reports_the_path_it_looked_at()
        {
            string output = Run("cheat", "load", "absent");

            Assert.Contains("no readable cheat file", output);
            Assert.Contains(Path.Combine(_dir, "cheats", "absent.json"), output);
        }

        [Fact]
        public void Files_lists_saved_sets_and_says_so_when_there_are_none()
        {
            Assert.Contains("No cheat files", Run("cheat", "files"));

            Run("cheat", "save", "zelda");

            Assert.Contains("zelda", Run("cheat", "files"));
        }

        [Theory]
        [InlineData("..")]
        [InlineData("../escape")]
        public void A_name_that_could_escape_the_cheats_directory_is_refused(string name)
        {
            Assert.Contains("not a usable file name", Run("cheat", "save", name));
            Assert.Contains("not a usable file name", Run("cheat", "load", name));
        }

        [Fact]
        public void Save_and_load_appear_in_the_usage_text()
        {
            Assert.Contains("cheat save", _command.Usage);
            Assert.Contains("cheat load", _command.Usage);
            Assert.Contains("cheat files", _command.Usage);
        }
    }
}
