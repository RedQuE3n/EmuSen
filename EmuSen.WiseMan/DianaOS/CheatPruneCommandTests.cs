using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // `cheat db prune` - see `man cheat`.
    [Collection(TestCollections.ProcessGlobals)]
    public class CheatPruneCommandTests : IDisposable
    {
        private readonly string _root;

        public CheatPruneCommandTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenPruneCmd", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings { CheatDatabaseDirectory = _root }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void WriteSystem(string system, int files)
        {
            string dir = Path.Combine(_root, system);
            Directory.CreateDirectory(dir);
            for (int i = 0; i < files; i++) File.WriteAllText(Path.Combine(dir, $"Game {i}.cht"), "cheats = 0\n");
        }

        private static DianaOSInterpreter NewShell()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return DianaOSInterpreter.CreateDefault(target, null,
                new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec(),
                supportedCheatSystems: () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);
        }

        // Without --apply nothing may be touched, however clear the plan is.
        [Fact]
        public void Prune_without_apply_only_reports()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            WriteSystem("Sony - PlayStation", 3);

            string output = NewShell().Execute("cheat db prune");

            Assert.Contains("Would delete 1 system(s), 3 file(s)", output);
            Assert.Contains("--apply", output);
            Assert.True(Directory.Exists(Path.Combine(_root, "Sony - PlayStation")));
        }

        [Fact]
        public void Prune_with_apply_deletes_and_says_what_it_kept()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            WriteSystem("Sony - PlayStation", 3);

            string output = NewShell().Execute("cheat db prune --apply");

            Assert.Contains("Deleted 1 system(s), 3 file(s)", output);
            Assert.Contains("Nintendo - Super Nintendo Entertainment System", output);
            Assert.False(Directory.Exists(Path.Combine(_root, "Sony - PlayStation")));
            Assert.True(Directory.Exists(Path.Combine(_root, "Nintendo - Super Nintendo Entertainment System")));
        }

        [Fact]
        public void Prune_on_an_already_pruned_database_says_so()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);

            Assert.Contains("Nothing to prune", NewShell().Execute("cheat db prune"));
        }

        // A shell built without the injection prunes nothing rather than
        // everything - see §4.16.
        [Fact]
        public void A_shell_told_nothing_about_cores_prunes_nothing()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 2);
            WriteSystem("Sony - PlayStation", 3);

            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            string output = DianaOSInterpreter.CreateDefault(target).Execute("cheat db prune");

            Assert.Contains("Nothing to prune", output);
            Assert.True(Directory.Exists(Path.Combine(_root, "Sony - PlayStation")));
        }

        [Fact]
        public void Prune_is_listed_and_suggested_like_the_other_db_subcommands()
        {
            Assert.Contains("prune", NewShell().Execute("cheat db prun"));
        }
    }
}
