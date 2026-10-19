using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // Part (c) of the parity class: the session service's functions through each implementation's twins and its tables, every answer and the files left compared - see EmuSen_Backend.md §29.21.
    public partial class CrystalParityTests
    {
        private static string TwinScriptPath(string name) => Path.Combine(AppContext.BaseDirectory, "Crystal", "TwinScripts", name + ".script");

        // A twin script run by each implementation in turn over the same cores and planted files, then the Saves tree it left.
        private void SameTwins(string script, IReadOnlyList<string> cores, IReadOnlyDictionary<string, string> values, Action? plant = null) =>
            SameTwinText(File.ReadAllText(TwinScriptPath(script)), cores, values, plant);

        private void SameTwinText(string text, IReadOnlyList<string> cores, IReadOnlyDictionary<string, string> values, Action? plant = null)
        {
            bool logging = DebugLogging.MasterEnabled;
            var runs = new List<List<string>>();
            try
            {
                foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
                {
                    if (Directory.Exists(DataStore.Saves)) Directory.Delete(DataStore.Saves, recursive: true);
                    plant?.Invoke();
                    List<string> answers = Answers(impl, cores, (i, rt) =>
                    {
                        using var run = new CrystalTwinScript(i, rt, values);
                        return run.Run(text);
                    });
                    answers.Add(TreeOf(DataStore.Saves));
                    answers.Add(TreeOf(DataStore.Cheats));
                    answers.Add(TreeOf(Path.Combine(ConfigStore.Directory, "cheats")));
                    runs.Add(answers);
                }
            }
            finally { DebugLogging.MasterEnabled = logging; }
            for (int k = 0; k < runs[0].Count; k++) _output.WriteLine($"{k}: {runs[0][k]}");
            Compare(runs[0], runs[1]);
        }

        // The test core's games: a plain one, one with a processor that cannot halt, a second game.
        private Dictionary<string, string> TestGames()
        {
            string dir = Path.Combine(_root, "Games");
            Directory.CreateDirectory(dir);
            string G(string name, params byte[] payload)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, CoreAdapterTests.Image(payload));
                return path;
            }
            return new Dictionary<string, string>
            {
                ["GAME"] = G("game.tst", 1, 2, 3, 4, 5, 6, 7, 8),
                ["HELPER"] = G("helper.tst", 0xC2, 9, 8, 7),
                ["OTHER"] = G("other.tst", 5, 5),
                ["DIR"] = dir,
            };
        }

        // A game of each of the build's engines, as the debug parity class makes them.
        private Dictionary<string, string> EngineGames()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            byte[] loop = { 0xA2, 0x05, 0xA9, 0x10, 0x8D, 0x00, 0x20, 0x20, 0x0D, 0x80, 0x4C, 0x00, 0x80, 0xA5, 0x42, 0x60 };
            return new Dictionary<string, string>
            {
                ["NES"] = W("Loop.nes", SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, loop) })),
                ["SFC"] = W("Game.sfc", SyntheticRom.Build((5, new byte[] { 0xA9, 0x12, 0x8D, 0x42, 0x00, 0x20, 0x0F, 0x80, 0x80, 0xF6, 0x60 }))),
                ["GB"] = W("Game.gb", SyntheticGbRom.Build()),
                ["GBC"] = W("Game.gbc", SyntheticGbRom.Build(cgbFlag: 0x80)),
                ["N64"] = W("Game.z64", SyntheticN64Rom.Build(patches: (0, MarsDebugClaims.CountForever))),
                ["MD"] = W("Game.md", SyntheticMdRom.Cartridge()),
                ["PSX"] = W("Game.iso", SyntheticPsxDisc.Build().Iso),
            };
        }

        [Fact]
        public void The_session_run_control_memory_processors_and_states_answer_alike_on_the_build_s_engines()
        {
            Dictionary<string, string> games = EngineGames();
            string block = File.ReadAllText(TwinScriptPath("engine"));
            string text = string.Concat(games.Values.Select(g => block.Replace("$GAME", g)));
            SameTwinText(text, BuildTree, games);
        }

        [Fact]
        public void The_session_run_control_memory_processors_states_logs_and_games_answer_alike_on_the_test_core() =>
            SameTwins("machine", new[] { TestTree(plain: false) }, TestGames());

        [Fact]
        public void The_registries_answer_alike_on_the_test_core() =>
            SameTwins("registries", new[] { TestTree(plain: false) }, TestGames());
    
        [Fact]
        public void The_registries_answer_alike_on_MoonRT()
        {
            byte[] head =
            {
                0xA2, 0x05, 0xA9, 0x10, 0x8D, 0x00, 0x20, 0xA9, 0x03, 0x8D, 0x05, 0x20, 0x8D, 0x05, 0x20, 0xA9, 0x21, 0x8D, 0x06, 0x20, 0xA9, 0x08, 0x8D, 0x06, 0x20,
                0xA9, 0xC0, 0x8D, 0x00, 0x40, 0xA9, 0x3F, 0x8D, 0x06, 0x20, 0xA9, 0x00, 0x8D, 0x06, 0x20, 0xA9, 0x21, 0x8D, 0x07, 0x20, 0xA9, 0x16, 0x8D, 0x07, 0x20,
                0xA9, 0x00, 0x8D, 0x03, 0x20, 0xA9, 0x40, 0x8D, 0x04, 0x20, 0xA9, 0x05, 0x8D, 0x04, 0x20, 0xA9, 0x41, 0x8D, 0x04, 0x20, 0xA9, 0x80, 0x8D, 0x04, 0x20,
                0xE6, 0x42,
            };
            int call = 0x8000 + head.Length, sub = call + 6;
            byte[] code = head.Concat(new byte[] { 0x20, (byte)sub, (byte)(sub >> 8), 0x4C, 0x00, 0x80, 0xA5, 0x42, 0x60 }).ToArray();
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string image = Path.Combine(dir, "Loop.nes");
            File.WriteAllBytes(image, SyntheticNesRom.Build(patches: new (int, byte[])[] { (0, code) }));
            SameTwins("moonrt-registries", BuildTree, new Dictionary<string, string>
            {
                ["IMAGE"] = image, ["W"] = "66", ["SUB"] = sub.ToString(), ["CALL"] = call.ToString(), ["INC"] = (call - 2).ToString(),
            });
        }
    
        [Fact]
        public void The_views_expressions_trace_and_telemetry_answer_alike_on_the_test_core_and_the_build_s_engines()
        {
            Dictionary<string, string> games = EngineGames();
            var spaces = new Dictionary<string, (string Map, string Tiles)>
            {
                ["NES"] = ("CIRAM", "CHR"), ["SFC"] = ("VRAM", "VRAM"), ["GB"] = ("VRAM", "VRAM"), ["GBC"] = ("VRAM", "VRAM"), ["N64"] = ("RDRAM", "RDRAM"), ["MD"] = ("VRAM", "VRAM"), ["PSX"] = ("VRAM", "VRAM"),
            };
            string block = File.ReadAllText(TwinScriptPath("views"));
            string text = string.Concat(games.Select(g => block.Replace("$GAME", g.Value).Replace("$TSPACE", spaces[g.Key].Map).Replace("$PSPACE", spaces[g.Key].Tiles)));
            SameTwinText(text, BuildTree, games);
            Dictionary<string, string> test = TestGames();
            SameTwinText(block.Replace("$GAME", test["GAME"]).Replace("$TSPACE", "RAM").Replace("$PSPACE", "RAM"), new[] { TestTree(plain: false) }, test);
        }
    
        // A cheat database of three systems and a saved list, planted before each implementation runs.
        private static void PlantCheats()
        {
            if (Directory.Exists(DataStore.Cheats)) Directory.Delete(DataStore.Cheats, recursive: true);
            string cheats = Path.Combine(ConfigStore.Directory, "cheats");
            if (Directory.Exists(cheats)) Directory.Delete(cheats, recursive: true);
            void W(string system, string game, string text)
            {
                string dir = Path.Combine(DataStore.Cheats, system);
                Directory.CreateDirectory(dir);
                File.WriteAllText(Path.Combine(dir, game + ".cht"), text);
            }
            W("Nintendo - Nintendo Entertainment System", "Loop", "cheats = 2\ncheat0_desc = \"Lives\"\ncheat0_code = \"SXIOPO\"\ncheat0_enable = true\ncheat1_desc = \"Raw\"\ncheat1_address = 66\ncheat1_value = 9\ncheat1_handler = 1\ncheat1_memory_search_size = 3\ncheat1_cheat_type = 1\n");
            W("Nintendo - Nintendo Entertainment System", "Other Game", "cheats = 0\n");
            W("Nintendo - Super Nintendo Entertainment System", "Loop Plus", "cheats = 1\ncheat0_desc = \"Par\"\ncheat0_code = \"7E0DBF63\"\n");
            W("Unclaimed - Nobody", "Stray", "cheats = 0\n");
        }

        [Fact]
        public void The_cheats_answer_alike_on_the_test_core_and_the_build_s_engines()
        {
            Dictionary<string, string> games = EngineGames();
            string block = File.ReadAllText(TwinScriptPath("cheats"));
            var rows = new[]
            {
                (games["NES"], "RAM", "0042:09", "SXIOPO", "Nintendo - Nintendo Entertainment System"),
                (games["SFC"], "CpuBus", "7E0DBF63", "C264-64D7", "Nintendo - Super Nintendo Entertainment System"),
                (games["GB"], "CPUBUS", "010F04C1", "00A-17B-C49", "Nintendo - Game Boy"),
                (games["MD"], "CpuBus", "FFFE00:0063", "ATBT-AA4L", "Sega - Mega Drive - Genesis"),
            };
            string text = string.Concat(rows.Select(r => block.Replace("$GAME", r.Item1).Replace("$SPACE", r.Item2).Replace("$CODE", r.Item3).Replace("$GENIE", r.Item4).Replace("$CHEATSYSTEM", r.Item5)));
            SameTwinText(text, BuildTree, games, PlantCheats);
            Dictionary<string, string> test = TestGames();
            SameTwinText(block.Replace("$GAME", test["GAME"]).Replace("$SPACE", "RAM").Replace("$CODE", "0042:09").Replace("$GENIE", "SXIOPO").Replace("$CHEATSYSTEM", "Nintendo - Nintendo Entertainment System"),
                new[] { TestTree(plain: false) }, test, PlantCheats);
            SameTwins("cheats-nogame", BuildTree, games, PlantCheats);
        }
    
        [Fact]
        public void The_recorder_answers_alike_through_the_api()
        {
            Dictionary<string, string> games = EngineGames();
            string rec = Path.Combine(_root, "Recordings");
            void Clear() { if (Directory.Exists(rec)) Directory.Delete(rec, recursive: true); }
            SameTwinText(File.ReadAllText(TwinScriptPath("recorder")).Replace("$GAME", games["NES"]).Replace("$REC", rec), BuildTree, games, Clear);
        }
    }
}
