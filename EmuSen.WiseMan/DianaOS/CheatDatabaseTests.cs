using System;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.DianaOS
{
    // The cheat database: using a tree the user already has, and fetching
    // one on request. EmuSen redistributes no cheat data - see `man cheat`.
    [Collection(TestCollections.ProcessGlobals)]
    public class CheatDatabaseTests : IDisposable
    {
        private readonly string _root;
        private readonly string _dbDir;

        public CheatDatabaseTests()
        {
            _root = Path.Combine(Path.GetTempPath(), "EmuSenCheatDbTests", Guid.NewGuid().ToString("N"));
            _dbDir = Path.Combine(_root, "Cheats");
            Directory.CreateDirectory(_dbDir);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings { CheatDatabaseDirectory = _dbDir }.Save();
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

        private void WriteChtFile(string system, string game, string contents = OneCheat)
        {
            string dir = Path.Combine(_dbDir, system);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, game + ".cht"), contents);
        }

        private static DianaOSInterpreter NewShell(out SnesDebugTarget target)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return DianaOSInterpreter.CreateDefault(target, null,
                new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec(),
                new EmuSen.Cores.Nintendo.Venus.Cheats.GameGenieCheatCodec());
        }

        // --- Option 1: a tree the user already has ---

        [Fact]
        public void An_existing_cheats_folder_is_indexed_by_system_and_game()
        {
            WriteChtFile("Nintendo - Super Nintendo Entertainment System", "Super Mario World (USA)");
            WriteChtFile("Nintendo - Super Nintendo Entertainment System", "Super Metroid (Japan, USA)");
            WriteChtFile("Sony - PlayStation", "Final Fantasy VII (USA) (Disc 1)");

            var db = new CheatDatabase(_dbDir);

            Assert.True(db.Exists);
            Assert.Equal(3, db.All().Count);

            var systems = db.Systems();
            Assert.Equal(2, systems.Count);
            Assert.Equal(2, systems.Single(s => s.System.StartsWith("Nintendo")).Count);
        }

        [Fact]
        public void A_missing_directory_reports_empty_rather_than_throwing()
        {
            var db = new CheatDatabase(Path.Combine(_root, "not-here"));

            Assert.False(db.Exists);
            Assert.Empty(db.All());
            Assert.Empty(db.Systems());
            Assert.Null(db.BestMatch("anything"));
        }

        [Theory]
        [InlineData("Super Mario World (USA)")]   // exact
        [InlineData("Super Mario World (USA).sfc")] // a loaded ROM's file name
        [InlineData("super mario world (usa)")]   // case
        [InlineData("Super Mario")]               // prefix
        [InlineData("Mario World")]               // substring
        public void A_game_is_found_the_ways_someone_would_actually_type_it(string query)
        {
            WriteChtFile("SNES", "Super Mario World (USA)");

            Assert.Equal("Super Mario World (USA)", new CheatDatabase(_dbDir).BestMatch(query)?.Game);
        }

        // A ROM loads as "Game (USA).sfc" but its cheat file is "Game (USA).cht".
        [Fact]
        public void An_exact_match_outranks_a_mere_substring()
        {
            WriteChtFile("SNES", "Super Mario World 2 - Yoshi's Island (USA)");
            WriteChtFile("SNES", "Super Mario World (USA)");

            Assert.Equal("Super Mario World (USA)", new CheatDatabase(_dbDir).BestMatch("Super Mario World (USA)")?.Game);
        }

        [Fact]
        public void The_shell_finds_and_loads_a_game_from_the_database()
        {
            WriteChtFile("Nintendo - Super Nintendo Entertainment System", "Super Mario World (USA)");
            DianaOSInterpreter shell = NewShell(out SnesDebugTarget target);

            Assert.Contains("Super Mario World (USA)", shell.Submit("cheat db find mario").Output);

            string loaded = shell.Submit("cheat db load Super Mario World (USA).sfc").Output;

            Assert.Contains("Loaded 1 cheat(s)", loaded);
            Assert.Contains("all disabled", loaded);

            CheatInfo cheat = Assert.Single(target.Cheats.GetCheats());
            Assert.Equal("Infinite Maximum Coins", cheat.Description);
            Assert.False(cheat.Enabled);
            Assert.Equal(0x7E0DBF, cheat.Writes[0].Address);
        }

        [Fact]
        public void Status_reports_the_directory_and_its_contents()
        {
            WriteChtFile("SNES", "A Game");
            WriteChtFile("SNES", "Another Game");
            DianaOSInterpreter shell = NewShell(out _);

            string status = shell.Submit("cheat db").Output;

            Assert.Contains("2 cheat file(s)", status);
            Assert.Contains("SNES", status);
        }

        [Fact]
        public void An_empty_database_points_at_how_to_get_one()
        {
            Directory.Delete(_dbDir, recursive: true);
            DianaOSInterpreter shell = NewShell(out _);

            string status = shell.Submit("cheat db").Output;

            Assert.Contains("cheat db update", status);
        }

        [Fact]
        public void Loading_a_game_that_is_not_there_says_so()
        {
            WriteChtFile("SNES", "Super Mario World (USA)");
            DianaOSInterpreter shell = NewShell(out _);

            Assert.Contains("No matching cheat file", shell.Submit("cheat db load Chrono Trigger").Output);
        }

        // --- Option 2: fetching one on request ---

        private static Stream BuildZip(params (string Path, string Contents)[] entries)
        {
            var buffer = new MemoryStream();
            using (var archive = new ZipArchive(buffer, ZipArchiveMode.Create, leaveOpen: true))
            {
                foreach ((string path, string contents) in entries)
                {
                    ZipArchiveEntry entry = archive.CreateEntry(path);
                    using var writer = new StreamWriter(entry.Open(), Encoding.UTF8);
                    writer.Write(contents);
                }
            }
            buffer.Position = 0;
            return buffer;
        }

        [Fact]
        public void An_update_unpacks_every_cht_and_preserves_the_system_folders()
        {
            using Stream zip = BuildZip(
                ("cht/Nintendo - Super Nintendo Entertainment System/Super Mario World (USA).cht", OneCheat),
                ("cht/Sony - PlayStation/Final Fantasy VII (USA) (Disc 1).cht", OneCheat));

            CheatDatabaseInstallResult result = CheatDatabaseInstaller.Install(zip, _dbDir);

            Assert.Equal(2, result.Installed);
            Assert.Equal(0, result.Skipped);
            Assert.True(File.Exists(Path.Combine(_dbDir, "cht", "Sony - PlayStation", "Final Fantasy VII (USA) (Disc 1).cht")));
            Assert.Equal(2, new CheatDatabase(_dbDir).All().Count);
        }

        [Fact]
        public void Non_cheat_entries_in_the_archive_are_skipped()
        {
            using Stream zip = BuildZip(
                ("cht/SNES/Game.cht", OneCheat),
                ("README.md", "not a cheat"),
                ("cht/SNES/notes.txt", "also not"));

            CheatDatabaseInstallResult result = CheatDatabaseInstaller.Install(zip, _dbDir);

            Assert.Equal(1, result.Installed);
            Assert.Equal(2, result.Skipped);
        }

        // Zip-slip: an entry naming ../ must not escape the target.
        [Fact]
        public void An_archive_entry_cannot_write_outside_the_target_directory()
        {
            string escapee = Path.Combine(_root, "escaped.cht");
            using Stream zip = BuildZip(
                ("../escaped.cht", OneCheat),
                ("cht/SNES/Legit.cht", OneCheat));

            CheatDatabaseInstallResult result = CheatDatabaseInstaller.Install(zip, _dbDir);

            Assert.False(File.Exists(escapee), "a zip entry escaped the cheat database directory");
            Assert.Equal(1, result.Installed);
            Assert.Equal(1, result.Skipped);
        }

        [Fact]
        public void An_update_reinstalls_over_an_existing_file_rather_than_failing()
        {
            WriteChtFile("cht/SNES", "Game", "cheats = 0");

            using Stream zip = BuildZip(("cht/SNES/Game.cht", OneCheat));
            CheatDatabaseInstallResult result = CheatDatabaseInstaller.Install(zip, _dbDir);

            Assert.Equal(1, result.Installed);
            Assert.Contains("Infinite Maximum Coins", File.ReadAllText(Path.Combine(_dbDir, "cht", "SNES", "Game.cht")));
        }

        [Fact]
        public void The_update_command_reports_what_it_installed_and_credits_the_source()
        {
            CheatDatabaseInstaller.FetchOverride = _ => BuildZip(("cht/SNES/Game.cht", OneCheat));
            DianaOSInterpreter shell = NewShell(out _);

            string output = shell.Submit("cheat db update").Output;

            Assert.Contains("Installed 1 cheat file(s)", output);
            Assert.Contains("CC BY-SA 4.0", output);
            Assert.Contains("libretro", output);
            Assert.Contains("GameHacking.org", output);
            // The whole point of option 2: we host nothing.
            Assert.Contains("downloaded to your machine at your request", output);
        }

        [Fact]
        public void A_failed_download_reports_it_rather_than_throwing()
        {
            CheatDatabaseInstaller.FetchOverride = _ => throw new IOException("network is down");
            DianaOSInterpreter shell = NewShell(out _);

            string output = shell.Submit("cheat db update").Output;

            Assert.Contains("network is down", output);
        }

        [Fact]
        public void The_download_url_is_the_one_retroarchs_own_updater_uses()
        {
            Assert.Equal("https://buildbot.libretro.com/assets/frontend/cheats.zip", CheatDatabaseInstaller.LibretroCheatsUrl);
        }
    }
}
