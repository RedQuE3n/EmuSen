using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Crystal
{
    // The PlayStation 2 started empty and opened with a game in both runtimes: each refused in the firmware entry's words with no BIOS, started with one, and its card medium listed alike - see Sapphire_Native.md §30.15, §30.17 and §49.5.
    public partial class CrystalParityTests
    {
        [Fact]
        public void The_playstation_2_started_empty_or_with_a_game_is_refused_without_its_bios_and_lists_its_cards_the_same()
        {
            static string J(params (string Key, object Value)[] pairs) => System.Text.Json.JsonSerializer.Serialize(pairs.ToDictionary(p => p.Key, p => p.Value));
            string bios = Path.Combine(DataStore.Firmware, "ps2-0230a-20080220.bin");
            string game = Path.Combine(_root, "ps2", "Basalt Run.iso");
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            File.WriteAllBytes(game, EmuSen.WiseMan.Fixtures.SyntheticPs2Disc.Dvd().Iso);
            Same(BuildTree, (i, rt) =>
            {
                var a = new List<string>();
                if (Directory.Exists(DataStore.Saves)) Directory.Delete(DataStore.Saves, recursive: true);
                if (File.Exists(bios)) File.Delete(bios);
                a.Add(i.Open(rt, null, console: "PlayStation 2"));
                a.Add(i.Open(rt, game, flags: CrystalAbi.OpenNoBattery));
                a.Add(i.Read("sessions", rt));
                a.Add(i.CardsOp(rt, 0, J(("console", "PlayStation 2"))));
                Directory.CreateDirectory(DataStore.Firmware);
                // A stand-in of the BIOS's name and size, every byte zero, made here: no firmware is read.
                File.WriteAllBytes(bios, new byte[4 << 20]);
                string o = i.Open(rt, null, console: "PlayStation 2");
                a.AddRange(new[] { o, i.Read("session_info", rt, Id(o)), i.Read("save_media", rt, Id(o)), i.Call("close", rt, Id(o)) });
                string g = i.Open(rt, game, flags: CrystalAbi.OpenNoBattery);
                a.AddRange(new[] { g, i.Call("close", rt, Id(g)) });
                return a;
            });
        }
    }
}
