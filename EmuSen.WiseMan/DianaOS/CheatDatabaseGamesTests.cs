using System;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.DianaOS
{
    // Listing one system's games, and the shared .cht-into-a-registry path
    // behind `cheat import`, `cheat db load` and Mistress - see `man cheat`.
    public class CheatDatabaseGamesTests : IDisposable
    {
        private readonly string _dbDir;

        public CheatDatabaseGamesTests()
        {
            _dbDir = Path.Combine(Path.GetTempPath(), "EmuSenCheatGamesTests", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_dbDir);
        }

        public void Dispose()
        {
            try { if (Directory.Exists(_dbDir)) Directory.Delete(_dbDir, recursive: true); } catch { }
        }

        private const string Snes = "Nintendo - Super Nintendo Entertainment System";
        private const string Psx = "Sony - PlayStation";

        private const string TwoCheats = """
            cheats = 2

            cheat0_desc = "Infinite Maximum Coins"
            cheat0_code = "7E0DBF63"
            cheat0_enable = false

            cheat1_desc = "Infinite Lives"
            cheat1_code = "7E001963"
            cheat1_enable = true
            """;

        private void WriteChtFile(string system, string game, string contents = TwoCheats)
        {
            string dir = Path.Combine(_dbDir, system);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, game + ".cht"), contents);
        }

        private CheatDatabase Db() => new(_dbDir);

        private static EmuSen.DianaOS.DianaOS.Lib.ICheatCodeCodec Codec() =>
            new EmuSen.Cores.Nintendo.Venus.Cheats.ActionReplayCheatCodec();

        [Fact]
        public void Games_returns_only_the_named_systems_files_in_name_order()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            WriteChtFile(Snes, "Super Metroid (Japan, USA)");
            WriteChtFile(Snes, "Chrono Trigger (USA)");
            WriteChtFile(Psx, "Final Fantasy VII (USA) (Disc 1)");

            string[] games = Db().Games(Snes).Select(g => g.Game).ToArray();

            Assert.Equal(new[] { "Chrono Trigger (USA)", "Super Mario World (USA)", "Super Metroid (Japan, USA)" }, games);
        }

        [Fact]
        public void Games_of_an_unknown_system_is_empty_rather_than_everything()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");

            Assert.Empty(Db().Games("Sega - Mega Drive"));
        }

        // A real system folder holds thousands, so the filter carries the UI.
        [Fact]
        public void Games_narrows_on_a_case_insensitive_substring()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            WriteChtFile(Snes, "Super Metroid (Japan, USA)");
            WriteChtFile(Snes, "Chrono Trigger (USA)");

            Assert.Equal(2, Db().Games(Snes, "super").Count);
            Assert.Equal("Chrono Trigger (USA)", Db().Games(Snes, "CHRONO").Single().Game);
            Assert.Empty(Db().Games(Snes, "zelda"));
        }

        [Fact]
        public void A_blank_filter_is_no_filter()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            WriteChtFile(Snes, "Chrono Trigger (USA)");

            Assert.Equal(2, Db().Games(Snes, "   ").Count);
            Assert.Equal(2, Db().Games(Snes, null).Count);
        }

        // Everything imports off, whatever the file's own enable flag said -
        // cheat1_enable is true above.
        [Fact]
        public void Importing_a_cht_adds_every_cheat_disabled()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            var registry = new CheatRegistry();

            CheatImportResult result = CheatImport.FromChtFile(registry, Db().Games(Snes).Single().Path, Codec());

            Assert.Equal(2, result.Loaded);
            Assert.Equal(0, result.Skipped);
            Assert.All(registry.GetCheats(), c => Assert.False(c.Enabled));
            Assert.Contains(registry.GetCheats(), c => c.Description == "Infinite Maximum Coins");
        }

        [Fact]
        public void Importing_without_replace_adds_to_whatever_is_loaded()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            var registry = new CheatRegistry();
            string path = Db().Games(Snes).Single().Path;

            CheatImport.FromChtFile(registry, path, Codec());
            CheatImport.FromChtFile(registry, path, Codec());

            Assert.Equal(4, registry.GetCheats().Count);
        }

        // What the GUI does, so picking the same game twice cannot double it.
        [Fact]
        public void Importing_with_replace_drops_the_previous_list_first()
        {
            WriteChtFile(Snes, "Super Mario World (USA)");
            var registry = new CheatRegistry();
            registry.AddRamPoke("WRAM", 0x9C, 0x63, "a hand-added one");
            string path = Db().Games(Snes).Single().Path;

            CheatImport.FromChtFile(registry, path, Codec(), replace: true);
            CheatImport.FromChtFile(registry, path, Codec(), replace: true);

            Assert.Equal(2, registry.GetCheats().Count);
            Assert.DoesNotContain(registry.GetCheats(), c => c.Description == "a hand-added one");
        }

        // An undecodable entry is counted, and the rest of the file survives.
        [Fact]
        public void An_unparseable_entry_is_skipped_and_reported()
        {
            WriteChtFile(Snes, "Broken (USA)", """
                cheats = 2

                cheat0_desc = "Fine"
                cheat0_code = "7E0DBF63"
                cheat0_enable = false

                cheat1_desc = "Nonsense"
                cheat1_code = "ZZZZZZZZ"
                cheat1_enable = false
                """);

            var registry = new CheatRegistry();
            CheatImportResult result = CheatImport.FromChtFile(registry, Db().Games(Snes).Single().Path, Codec());

            Assert.Equal(1, result.Loaded);
            Assert.Equal(1, result.Skipped);
        }
    }
}
