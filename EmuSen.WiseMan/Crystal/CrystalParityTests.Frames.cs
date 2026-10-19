using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Crystal.Client.Contract;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 3 of the parity class: the frame, its picture and sound, the controls and the speed, every frame's answers compared - see EmuSen_Backend.md §29.16.
    public partial class CrystalParityTests
    {
        [Fact]
        public void The_test_core_s_frames_pictures_sound_controls_and_speeds_are_the_same()
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
                a.AddRange(new[] { i.Frame(rt, s, size: 100), i.Frame(rt, s, size: 104), i.Frame(rt, 99), i.Phases(rt, s), i.Picture(rt, s, len: 3) });
                for (int f = 0; f < 400; f++)
                {
                    if (f % 7 == 0) a.Add(i.Button(rt, s, 0, 8, f % 14 == 0));
                    if (f % 11 == 0) a.Add(i.Axis(rt, s, (uint)(f % 3), 0, f / 100.0 - 2));
                    if (f == 50) a.Add(i.Connected(rt, s, 1, false));
                    if (f == 60) a.Add(i.Connected(rt, s, 1, true));
                    if (f == 100) a.AddRange(new[] { i.Speed(rt, s, 350, 400, 40, 0), i.Read("session_info", rt, s) });
                    if (f == 140) a.Add(i.SpeedMode(rt, s, CrystalAbi.SpeedSlow));
                    if (f == 160) a.Add(i.SpeedMode(rt, s, CrystalAbi.SpeedTurbo));
                    if (f == 180) a.AddRange(new[] { i.Speed(rt, s, 0, 20, 500, CrystalAbi.SpeedDrawNone), i.SpeedMode(rt, s, CrystalAbi.SpeedNormal) });
                    if (f == 200) a.Add(i.Speed(rt, s, 100, 300, 25, 0));
                    if (f == 220) a.AddRange(new[] { i.DiscLid(rt, s, true), i.DiscInsert(rt, s, other), i.Read("session_info", rt, s) });
                    if (f == 290) a.Add(i.Read("session_info", rt, s));
                    a.Add(i.Frame(rt, f % 5 == 0 ? 0 : s));
                    if (f % 3 == 0) a.Add(i.Picture(rt, s));
                    if (f % 4 == 0) a.Add(i.Audio(rt, s, f % 8 == 0 ? 64 : 4096));
                }
                a.AddRange(new[]
                {
                    i.Button(rt, s, 0, 16, true), i.Button(rt, s, 9, 8, true), i.Axis(rt, s, 0, 6, 0), i.Axis(rt, s, 0, 0, double.NaN), i.Axis(rt, s, 0, 4, 3),
                    i.Speed(rt, s, 100, 300, 25, 2), i.SpeedMode(rt, s, 3), i.Wait(rt, s, 0), i.Phases(rt, s), i.Audio(rt, s, 0), i.Picture(rt, 77),
                    i.Read("session_info", rt, s), i.Call("close", rt, s), i.Frame(rt, s), i.Picture(rt, s), i.Audio(rt, s), i.Wait(rt, s, 0),
                });
                uint single = Id(i.Open(rt, plain, flags: CrystalAbi.OpenSingleRows | CrystalAbi.OpenNoBattery));
                for (int f = 0; f < 30; f++) a.AddRange(new[] { i.Frame(rt, single), i.Picture(rt, single) });
                a.Add(i.Call("close", rt, single));
                return a;
            });
        }

        [Fact]
        public void The_build_s_engines_run_frames_the_same()
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
                W("Colour.gbc", SyntheticGbRom.Build(cgbFlag: 0xC0)), W("Saves.z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), W("Game.md", SyntheticMdRom.Cartridge(saveBytes: 0x2000)),
                W("SegaCd.iso", SyntheticMdRom.Disc("U")), W("Psx.iso", SyntheticPsxDisc.Build().Iso),
            };
            SameWithFiles(BuildTree, () => { }, (i, rt) =>
            {
                var a = new List<string>();
                foreach (string game in games)
                {
                    string o = i.Open(rt, game);
                    uint s = Id(o);
                    a.AddRange(new[] { o, i.Phases(rt, s) });
                    for (int f = 0; f < 120; f++)
                    {
                        if (f % 10 == 0) a.AddRange(new[] { i.Button(rt, s, 0, (uint)(f / 10 % 16), true), i.Axis(rt, s, 0, (uint)(f / 10 % 6), 0.5) });
                        a.Add(i.Frame(rt, s));
                        if (f % 10 == 9) a.AddRange(new[] { i.Picture(rt, s), i.Audio(rt, s, 1 << 14) });
                    }
                    a.AddRange(new[] { i.Read("session_info", rt, s), i.Call("close", rt, s) });
                }
                string console = i.Open(rt, null, console: "PlayStation");
                for (int f = 0; f < 60; f++) a.AddRange(new[] { i.Frame(rt, Id(console)), i.Picture(rt, Id(console)), i.Audio(rt, Id(console), 1 << 14) });
                a.Add(i.Call("close", rt, Id(console)));
                return a;
            });
        }
    }
}
