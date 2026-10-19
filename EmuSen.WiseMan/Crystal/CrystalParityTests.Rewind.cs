using System.Collections.Generic;
using System.IO;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 4 of the parity class: rewind's chain, its reel and pictures, its steps and jumps, every answer compared - see EmuSen_Backend.md §29.17.
    public partial class CrystalParityTests
    {
        [Fact]
        public void The_test_core_s_rewind_steps_jumps_and_reels_the_same()
        {
            string tree = TestTree(plain: false);
            string dir = Path.Combine(_root, "Games");
            Directory.CreateDirectory(dir);
            string plain = Path.Combine(dir, "plain.tst"), other = Path.Combine(dir, "other.tst");
            File.WriteAllBytes(plain, CoreAdapterTests.Image(1, 2, 3, 4));
            File.WriteAllBytes(other, CoreAdapterTests.Image(5, 5));
            SameWithFiles(new[] { tree }, () => { }, (i, rt) =>
            {
                var a = new List<string>();
                uint s = Id(i.Open(rt, plain));
                string Reel() => i.Read("rewind_reel", rt, s);
                void Run(int frames)
                {
                    for (int f = 0; f < frames; f++)
                    {
                        if (f % 9 == 0) a.Add(i.Button(rt, s, 0, 8, f % 18 == 0));
                        a.Add(i.Frame(rt, s));
                        if (f % 5 == 0) a.Add(i.Picture(rt, s));
                    }
                }
                a.AddRange(new[] { Reel(), i.RewindStep(rt, s, 3), i.RewindTo(rt, s, 0), i.RewindSet(rt, s, true, 0, 0, 64), Reel() });
                Run(120);
                a.AddRange(new[] { Reel(), i.RewindPicture(rt, s, 0), i.RewindPicture(rt, s, 4), i.RewindPicture(rt, s, 100), i.RewindPicture(rt, s, 3), i.RewindPicture(rt, s, 9999) });
                a.AddRange(new[] { i.RewindStep(rt, s, 5), Reel(), i.Picture(rt, s), i.Audio(rt, s), i.RewindStep(rt, s, 1, size: 100), i.RewindStep(rt, s, 1, size: 104) });
                Run(20);
                a.AddRange(new[] { i.RewindTo(rt, s, 40), Reel(), i.RewindTo(rt, s, 41), i.Picture(rt, s) });
                a.AddRange(new[] { i.DiscLid(rt, s, true), i.DiscInsert(rt, s, other), i.Read("session_info", rt, s) });
                Run(80);
                a.AddRange(new[] { i.Read("session_info", rt, s), i.RewindTo(rt, s, 40), i.Read("session_info", rt, s), i.Picture(rt, s) });
                a.AddRange(new[] { i.RewindSet(rt, s, true, 2, 300, 32), Reel() });
                Run(60);
                a.AddRange(new[] { Reel(), i.RewindStep(rt, s, 1000), Reel(), i.RewindStep(rt, s, 1) });
                Run(10);
                a.AddRange(new[] { i.Call("reset", rt, s), Reel() });
                Run(10);
                a.AddRange(new[] { Reel(), i.RewindSet(rt, s, false, 0, 0, 0), Reel(), i.RewindStep(rt, s, 1), i.RewindSet(rt, 99, true, 0, 0, 0), i.Read("rewind_reel", rt, 99) });
                a.Add(i.Call("close", rt, s));
                return a;
            });
        }

        [Fact]
        public void The_build_s_engines_rewind_the_same()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            var games = new List<string>
            {
                W("Battery.nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 0, mapper: 1, battery: true)), W("Game.sfc", SyntheticRom.Build()), W("Mono.gb", SyntheticGbRom.Build()),
                W("Saves.z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), W("Game.md", SyntheticMdRom.Cartridge(saveBytes: 0x2000)), W("Psx.iso", SyntheticPsxDisc.Build().Iso),
            };
            SameWithFiles(BuildTree, () => { }, (i, rt) =>
            {
                var a = new List<string>();
                foreach (string game in games)
                {
                    uint s = Id(i.Open(rt, game));
                    a.Add(i.RewindSet(rt, s, true, 3, 0, 100));
                    for (int f = 0; f < 60; f++)
                    {
                        if (f % 10 == 0) a.Add(i.Button(rt, s, 0, (uint)(f / 10 % 16), true));
                        a.Add(i.Frame(rt, s));
                    }
                    a.AddRange(new[] { i.Read("rewind_reel", rt, s), i.RewindPicture(rt, s, 30), i.RewindStep(rt, s, 4), i.Picture(rt, s), i.RewindTo(rt, s, 9), i.Picture(rt, s) });
                    for (int f = 0; f < 10; f++) a.Add(i.Frame(rt, s));
                    a.AddRange(new[] { i.Picture(rt, s), i.Read("rewind_reel", rt, s), i.Call("close", rt, s) });
                }
                return a;
            });
        }
    }
}
