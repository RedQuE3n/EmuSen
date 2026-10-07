using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Players 3 and up on the Genesis, through the path a frontend takes: the stored port settings, CoreFactory, and each player's buttons by port - see Nephrite_Native.md §45.
    [Collection(TestCollections.ProcessGlobals)]
    public class NephritePlayersTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNephritePlayers_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;
        private readonly ITestOutputHelper _output;

        public NephritePlayersTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // The Genesis as a frontend makes it, the two port rows stored first.
        private CoreEngine Load(string rom, string pad1, string pad2 = "md.pad3")
        {
            var config = new GraphicsConfig();
            config.SetValue("Genesis", "pad1", pad1);
            config.SetValue("Genesis", "pad2", pad2);
            config.Save();
            return Assert.IsType<CoreEngine>(CoreFactory.Load(rom).Core);
        }

        [Fact]
        public void The_port_settings_say_each_players_pad()
        {
            static string Of(string a, string b) => string.Join(" ", GenesisSystems.PlayerControllers(k => k == "pad1" ? a : b).Select(c => c[^1]));
            Assert.Equal("3 3", Of("md.pad3", "md.pad3"));
            Assert.Equal("6 6 6 6 3", Of("md.teamplayer6", "md.pad3"));
            Assert.Equal("6 3 3 3 3", Of("md.pad6", "md.teamplayer3"));
            Assert.Equal("3 3 3 3 6 6 6 6", Of("md.teamplayer3", "md.teamplayer6"));
            Assert.Equal("3 3 3 3", Of("md.4way3", "md.pad6"));
            Assert.Equal("3 3", Of("md.pad3", "md.4way3"));
            Func<string, string?> stored = k => k == "pad1" ? "md.teamplayer6" : null;
            Assert.Equal((5, 2, 8), (ControllerPorts.ForConsole("Genesis", stored), ControllerPorts.ForConsole("Genesis", _ => null), ControllerPorts.ForConsole("Genesis")));
            Assert.Equal(("md.pad6", "md.pad3", null), (CoreCatalog.ControllerFor("Genesis", 3, stored), CoreCatalog.ControllerFor("Genesis", 4, stored), CoreCatalog.ControllerFor("Genesis", 5, stored)));
        }

        private static void Put(byte[] r, int at, IEnumerable<ushort> words)
        {
            foreach (ushort w in words)
            {
                r[at++] = (byte)(w >> 8);
                r[at++] = (byte)w;
            }
        }

        // A cartridge that reads a Team Player on port 1 as Plutiedev's page has it and stores each nibble from $FF0000, every frame's worth in a loop.
        internal static byte[] TeamPlayerReader()
        {
            var r = SyntheticMdRom.Cartridge();
            Put(r, 0, new ushort[] { 0x00FF, 0xFE00, 0x0000, 0x0200 });
            var code = new List<ushort> { 0x46FC, 0x2700 };
            void Step(ushort value, int store)
            {
                code.AddRange(new ushort[] { 0x13FC, value, 0x00A1, 0x0003, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x13F9, 0x00A1, 0x0003, 0x00FF, (ushort)store });
            }
            code.AddRange(new ushort[] { 0x13FC, 0x0060, 0x00A1, 0x0009 });
            int loop = 0x200 + 2 * code.Count;
            Step(0x60, 0x100);
            Step(0x20, 0x101);
            for (int n = 0; n < 18; n++) Step((ushort)(n % 2 == 0 ? 0x00 : 0x20), n);
            Step(0x60, 0x102);
            code.AddRange(new ushort[] { 0x4EF9, 0x0000, (ushort)loop });
            Put(r, 0x200, code);
            return r;
        }

        // Four players' buttons reach four pads of a Team Player: each pad's nibbles in the adapter's packet are that player's, by the RetroPad's mapping.
        [Fact]
        public void Four_players_reach_a_team_players_four_pads()
        {
            string rom = Path.Combine(_root, "reader.md");
            File.WriteAllBytes(rom, TeamPlayerReader());
            using var core = Load(rom, "md.teamplayer6");
            Assert.Equal(new[] { "md.pad6", "md.pad6", "md.pad6", "md.pad6", "md.pad3" }, core.Machine.Info.Ports.Select(p => p.Controller));
            Assert.Equal(5, ControllerPorts.Of(core));
            core.SetButton(0, PadButton.Up, true);
            core.SetButton(1, PadButton.Y, true);
            core.SetButton(2, PadButton.L, true);
            core.SetButton(2, PadButton.Left, true);
            core.SetButton(3, PadButton.Start, true);
            core.SetButton(3, PadButton.Select, true);
            for (int f = 0; f < 4; f++) core.RunFrame();
            byte[] got = Enumerable.Range(0, 18).Select(n => (byte)(core.ReadSpace("WRAM", n) & 0xF)).ToArray();
            Assert.Equal(new byte[] { 0, 0, 1, 1, 1, 1 }, got[..6]);
            Assert.Equal(new byte[] { 0xE, 0xF, 0xF }, got[6..9]);
            Assert.Equal(new byte[] { 0xF, 0xB, 0xF }, got[9..12]);
            Assert.Equal(new byte[] { 0xB, 0xF, 0xB }, got[12..15]);
            Assert.Equal(new byte[] { 0xF, 0x7, 0x7 }, got[15..18]);
        }

        // A port's row changed while the game runs plugs the adapter in between frames, and the new players' buttons are held from then.
        [Fact]
        public void An_adapter_plugged_in_while_the_game_runs_takes_its_players()
        {
            string rom = Path.Combine(_root, "reader.md");
            File.WriteAllBytes(rom, TeamPlayerReader());
            using var core = Load(rom, "md.pad3");
            core.RunFrame();
            Assert.Equal(2, ControllerPorts.Of(core));
            core.Set("pad1", "md.teamplayer3");
            core.RunFrame();
            Assert.Equal(5, ControllerPorts.Of(core));
            core.SetButton(3, PadButton.B, true);
            for (int f = 0; f < 4; f++) core.RunFrame();
            Assert.Equal((0x0, 0xF, 0xE), (core.ReadSpace("WRAM", 5) & 0xF, core.ReadSpace("WRAM", 12) & 0xF, core.ReadSpace("WRAM", 13) & 0xF));
        }

        // padprotocol.py's 4 Way Play cartridge: each write to a port register, then a read of port 1 stored from $FF0000, and $A5 at $FF0FFE when done.
        private static byte[] FourWayProgram(out int steps)
        {
            const ushort A = 0x03, B = 0x05, ControlA = 0x09, ControlB = 0x0B;
            var list = new List<(ushort Register, ushort Value, ushort Read)> { (ControlA, 0x40, A), (ControlB, 0x7F, A), (A, 0x40, A) };
            foreach (ushort choice in new ushort[] { 0x0C, 0x1C, 0x2C, 0x3C, 0x4C, 0x5C, 0x6C, 0x7C }) list.AddRange(new[] { (B, choice, A), (A, (ushort)0x40, A), (A, (ushort)0x00, A), (A, (ushort)0x40, A) });
            list.AddRange(new (ushort, ushort, ushort)[] { (B, 0x7C, B), (B, 0x0C, B), (B, 0x00, A), (B, 0x70, A), (B, 0x7F, A), (ControlB, 0x00, A), (ControlB, 0x7F, A), (B, 0x0C, A) });
            list.AddRange(new (ushort, ushort, ushort)[] { (ControlB, 0x40, A), (B, 0x00, A), (B, 0x10, A), (B, 0x20, A), (B, 0x30, A), (B, 0x40, A) });
            var r = SyntheticMdRom.Cartridge();
            Put(r, 0, new ushort[] { 0x00FF, 0xFE00, 0x0000, 0x0200 });
            var code = new List<ushort> { 0x46FC, 0x2700 };
            for (int n = 0; n < list.Count; n++) code.AddRange(new ushort[] { 0x13FC, list[n].Value, 0x00A1, list[n].Register, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x13F9, 0x00A1, list[n].Read, 0x00FF, (ushort)n });
            code.AddRange(new ushort[] { 0x13FC, 0x00A5, 0x00FF, 0x0FFE, 0x60FE });
            Put(r, 0x200, code);
            steps = list.Count;
            return r;
        }

        // The 4 Way Play step by step as Genesis Plus GX answers padprotocol.py's cartridge, but for the four steps Nephrite_Native.md §46.1 names; the last six are a game that leaves port 2's choosing lines inputs.
        [Fact]
        public void A_4_way_play_answers_the_protocol_cartridge_and_gives_player_1_when_its_choosing_lines_are_not_driven()
        {
            string rom = Path.Combine(_root, "fourway.md");
            File.WriteAllBytes(rom, FourWayProgram(out int steps));
            using var core = Load(rom, "md.4way3");
            core.SetButton(0, PadButton.Up, true);
            core.SetButton(1, PadButton.B, true);
            core.SetButton(2, PadButton.Left, true);
            core.SetButton(2, PadButton.Y, true);
            core.SetButton(3, PadButton.Start, true);
            for (int f = 0; f < 4; f++) core.RunFrame();
            Assert.Equal(0xA5, core.ReadSpace("WRAM", 0xFFE));
            string reference = "32 32 7E 7E 7E 32 7E 6F 6F 33 6F 7B 7B 23 7B 7F 7F 13 7F 7C 7C 3C 7C 7C 7C 3C 7C 7C 7C 3C 7C 7C 7C 3C 7C 7C 0C 7E 7C 7C 7C 7C 7E 7E 7E 7E 7E 7E 7E";
            string[] expected = reference.Split(' ');
            // The data latch at power-on is $7F here and 0 there (steps 0 to 2), and a choice does not outlast its lines being driven (step 40).
            (expected[0], expected[1], expected[2], expected[40]) = ("7E", "7C", "7C", "7E");
            Assert.Equal(expected, Enumerable.Range(0, steps).Select(n => core.ReadSpace("WRAM", n).ToString("X2")));
        }

        public const string GamesVariable = "EMUSEN_NEPHRITE_GAMES";

        // Two games of the tester's library with four players, each reading its four pads where Genesis Plus GX, run as a black box, has it read them - see Nephrite_Native.md §45.3.
        [Theory]
        [InlineData("NBA Jam (EU) (REV 01) [!].bin", "md.teamplayer3", 0x02B5)]
        [InlineData("NBA Showdown 94 (UE) [!].bin", "md.4way3", 0x801D)]
        public void A_four_player_game_reads_four_pads(string game, string adapter, int first)
        {
            string? folder = Environment.GetEnvironmentVariable(GamesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, game)))
            {
                _output.WriteLine($"{GamesVariable} names no folder with {game}: not run");
                return;
            }
            string rom = Path.Combine(_root, "game.bin");
            File.Copy(Path.Combine(folder, game), rom);
            using var core = Load(rom, adapter);
            PadButton[] held = { PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right };
            for (int f = 0; f < 900; f++)
            {
                if (f == 200) for (int p = 0; p < 4; p++) core.SetButton(p, held[p], true);
                core.RunFrame();
            }
            Assert.Equal(new[] { 1, 2, 4, 8 }, Enumerable.Range(0, 4).Select(p => (int)core.ReadSpace("WRAM", first + 2 * p)));
        }

        // Three of the 63 games not made for the 4 Way Play that read player 1 through it, where Genesis Plus GX has them keep Up held - see Nephrite_Native.md §46.1.
        [Theory]
        [InlineData("Urban Strike (UEJ) [!].bin", 0x46E5)]
        [InlineData("James Pond 2 - Codename RoboCod (U) [!].bin", 0xB4CD)]
        [InlineData("John Madden Football 93 - Championship Edition (U) [!].bin", 0xD2CB)]
        public void A_game_not_made_for_the_4_way_play_reads_player_1_through_it(string game, int address)
        {
            string? folder = Environment.GetEnvironmentVariable(GamesVariable);
            if (folder is null || !File.Exists(Path.Combine(folder, game)))
            {
                _output.WriteLine($"{GamesVariable} names no folder with {game}: not run");
                return;
            }
            string rom = Path.Combine(_root, "game.bin");
            File.Copy(Path.Combine(folder, game), rom);
            var read = new List<int>();
            foreach (bool held in new[] { false, true })
            {
                using var core = Load(rom, "md.4way3");
                for (int f = 0; f < 900; f++)
                {
                    if (f == 200) core.SetButton(0, PadButton.Up, held);
                    core.RunFrame();
                }
                read.Add(core.ReadSpace("WRAM", address));
            }
            Assert.Equal(new[] { 0, 1 }, read);
        }
    }
}
