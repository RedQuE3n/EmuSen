using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // Dropping the systems no core can use - see EmuSen_Settings_Reference.md §4.16.
    public class CheatDatabasePrunerTests : IDisposable
    {
        private readonly string _root;

        public CheatDatabasePrunerTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenPruner", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private void WriteSystem(string system, int files)
        {
            string dir = Path.Combine(_root, system);
            Directory.CreateDirectory(dir);
            for (int i = 0; i < files; i++) File.WriteAllText(Path.Combine(dir, $"Game {i}.cht"), "cheats = 0\n");
        }

        private CheatDatabase Db() => new(_root);

        private static readonly string[] Snes = { "Nintendo - Super Nintendo Entertainment System" };

        [Fact]
        public void A_plan_keeps_what_a_core_claims_and_removes_the_rest()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 3);
            WriteSystem("Sega - Mega Drive - Genesis", 2);
            WriteSystem("Sony - PlayStation", 4);

            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Snes);

            Assert.True(plan.CanApply);
            Assert.Equal(Snes, plan.Keeping);
            Assert.Equal(new[] { "Sega - Mega Drive - Genesis", "Sony - PlayStation" },
                plan.Removing.Select(r => r.System).OrderBy(s => s));
            Assert.Equal(6, plan.Files);
            Assert.True(plan.Bytes > 0);
        }

        [Fact]
        public void Applying_deletes_only_the_unsupported_folders()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 3);
            WriteSystem("Sega - Mega Drive - Genesis", 2);

            CheatDatabase db = Db();
            (int removed, IReadOnlyList<string> failed) = CheatDatabasePruner.Apply(db, CheatDatabasePruner.Plan(db, Snes));

            Assert.Equal(1, removed);
            Assert.Empty(failed);
            Assert.True(Directory.Exists(Path.Combine(_root, "Nintendo - Super Nintendo Entertainment System")));
            Assert.False(Directory.Exists(Path.Combine(_root, "Sega - Mega Drive - Genesis")));
        }

        // The whole point of the guard: a build that has not filled
        // CheatSystems in yet must not read as "delete everything".
        [Fact]
        public void An_empty_keep_set_refuses_rather_than_deleting_everything()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 3);
            WriteSystem("Sega - Mega Drive - Genesis", 2);

            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Array.Empty<string>());

            Assert.False(plan.CanApply);
            Assert.Empty(plan.Removing);
            Assert.Contains("no core", plan.Reason!);

            CheatDatabase db = Db();
            Assert.Equal(0, CheatDatabasePruner.Apply(db, plan).Removed);
            Assert.True(Directory.Exists(Path.Combine(_root, "Sega - Mega Drive - Genesis")));
        }

        // A wrong folder or a wrong mapping looks exactly like "every system
        // here is unsupported", which is the one case worth refusing.
        [Fact]
        public void A_keep_set_matching_nothing_on_disk_refuses()
        {
            WriteSystem("Sega - Mega Drive - Genesis", 2);
            WriteSystem("Sony - PlayStation", 4);

            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Snes);

            Assert.False(plan.CanApply);
            Assert.Contains("refusing to delete the whole database", plan.Reason!);
        }

        [Fact]
        public void An_already_pruned_database_says_so_rather_than_planning_nothing()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 3);

            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Snes);

            Assert.False(plan.CanApply);
            Assert.Contains("already", plan.Reason!);
        }

        [Fact]
        public void An_empty_directory_refuses()
        {
            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Snes);

            Assert.False(plan.CanApply);
            Assert.Contains("no cheat files", plan.Reason!);
        }

        [Fact]
        public void System_names_match_case_insensitively()
        {
            WriteSystem("nintendo - super nintendo entertainment system", 3);
            WriteSystem("Sony - PlayStation", 1);

            CheatPrunePlan plan = CheatDatabasePruner.Plan(Db(), Snes);

            Assert.True(plan.CanApply);
            Assert.Equal("Sony - PlayStation", plan.Removing.Single().System);
        }

        // A symlinked system folder must not let a delete escape the tree.
        [Fact]
        public void A_symlinked_system_folder_is_refused_rather_than_followed()
        {
            WriteSystem("Nintendo - Super Nintendo Entertainment System", 1);

            string outside = Path.Combine(Path.GetTempPath(), "EmuSenPrunerOutside", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(outside);
            File.WriteAllText(Path.Combine(outside, "precious.cht"), "cheats = 0\n");

            try
            {
                Directory.CreateSymbolicLink(Path.Combine(_root, "Sony - PlayStation"), outside);
            }
            catch (Exception)
            {
                return; // no symlink permission here - nothing to assert
            }

            try
            {
                CheatDatabase db = Db();
                (int removed, IReadOnlyList<string> failed) = CheatDatabasePruner.Apply(db, CheatDatabasePruner.Plan(db, Snes));

                Assert.Equal(0, removed);
                Assert.Single(failed);
                Assert.True(File.Exists(Path.Combine(outside, "precious.cht")), "the link target must survive");
            }
            finally
            {
                try { Directory.Delete(outside, recursive: true); } catch { }
            }
        }

        // The agnostic half: the pruner is told what to keep, and what to
        // keep comes off the core registry rather than out of the pruner.
        [Fact]
        public void Supported_systems_come_off_the_core_registry_deduplicated()
        {
            var one = new CoreDescriptor("SNES (Venus)", new[] { ".sfc" }, new[] { "Nintendo - Super Nintendo Entertainment System", "Nintendo - Satellaview" });
            var registry = new Dictionary<string, CoreDescriptor> { ["venus"] = one, ["snes"] = one };

            IReadOnlyCollection<string> systems = CoreDescriptor.SupportedCheatSystems(registry.Values);

            Assert.Equal(2, systems.Count);
            Assert.Contains("Nintendo - Satellaview", systems);
        }

        [Fact]
        public void A_core_claiming_no_cheat_systems_contributes_none()
        {
            var descriptor = new CoreDescriptor("Someday (unimplemented)", new[] { ".xyz" });

            Assert.Empty(descriptor.CheatSystemNames);
            Assert.Empty(CoreDescriptor.SupportedCheatSystems(new[] { descriptor }));
        }

        // What this build actually ships, so filling in a second core's
        // folders is a test failure here rather than a silent wrong prune.
        [Fact]
        public void The_shipped_catalog_claims_the_snes_folders()
        {
            IReadOnlyCollection<string> systems = EmuSen.Cores.CoreCatalog.SupportedCheatSystems;

            Assert.Contains("Nintendo - Super Nintendo Entertainment System", systems);
            Assert.Contains("Nintendo - Satellaview", systems);
        }
    }
}
