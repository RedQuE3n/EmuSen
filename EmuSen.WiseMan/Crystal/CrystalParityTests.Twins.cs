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
    }
}
