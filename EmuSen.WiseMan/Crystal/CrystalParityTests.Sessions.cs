using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // Piece 2c of the parity class: a session's life, its discs and its save media, the answers and the files left compared - see EmuSen_Backend.md §29.15.
    public partial class CrystalParityTests
    {
        // Each file and its hash, a PNG's by its pixels, since the two encoders' bytes differ and their pictures do not.
        private static string TreeOf(string dir) => !Directory.Exists(dir) ? "" : string.Join("\n", Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
            .Select(f => $"{Path.GetRelativePath(dir, f)} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(f.EndsWith(".png") ? PixelsOf(f) : File.ReadAllBytes(f)))}"));

        private static byte[] PixelsOf(string png)
        {
            using var bitmap = SkiaSharp.SKBitmap.Decode(png);
            if (bitmap is null) return File.ReadAllBytes(png);
            using var rgba = bitmap.Copy(SkiaSharp.SKColorType.Rgba8888);
            return System.Text.Encoding.ASCII.GetBytes($"{rgba.Width}x{rgba.Height} ").Concat(rgba.Bytes).ToArray();
        }

        // Each implementation over the same planted files, its answers followed by the Saves and Unpacked trees it leaves.
        private void SameWithFiles(IReadOnlyList<string> cores, Action plant, Func<CrystalImpl, nint, IEnumerable<string>> asks, Func<string>? also = null)
        {
            var runs = new List<List<string>>();
            foreach (CrystalImpl impl in new[] { CrystalImpl.Managed, CrystalImpl.Native })
            {
                foreach (string d in new[] { DataStore.Saves, DataStore.Unpacked })
                    if (Directory.Exists(d)) Directory.Delete(d, recursive: true);
                plant();
                List<string> answers = Answers(impl, cores, asks);
                answers.Add(TreeOf(DataStore.Saves));
                answers.Add(TreeOf(DataStore.Unpacked));
                if (also is not null) answers.Add(also());
                runs.Add(answers);
            }
            for (int k = 0; k < runs[0].Count; k++) _output.WriteLine($"{k}: {runs[0][k]}");
            Compare(runs[0], runs[1]);
        }

        private static uint Id(string opened) => uint.TryParse(opened, out uint id) ? id : 999;

        private static byte[] Bytes(int count, Func<int, int> at) => Enumerable.Range(0, count).Select(i => (byte)at(i)).ToArray();

        [Fact]
        public void A_test_core_s_sessions_open_change_discs_reset_and_close_the_same()
        {
            string tree = TestTree(plain: false);
            string dir = Path.Combine(_root, "Games");
            Directory.CreateDirectory(dir);
            string G(string name, params byte[] payload)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, CoreAdapterTests.Image(payload));
                return path;
            }
            string plain = G("plain.tst", 1, 2, 3, 4), card = G("card.tst", 0xCD, 9), memory = G("memory.tst", 0xCE, 8), cart = G("cart.tst", 0xCA, 7);
            string boot = G("boot.tst", 0xB0, 6), other = G("other.tst", 5, 5);
            string zip = Path.Combine(dir, "zipped.zip");
            using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
            using (var entry = archive.CreateEntry("inside.tst").Open()) entry.Write(CoreAdapterTests.Image(3, 3));
            void Plant()
            {
                string saves = Path.Combine(DataStore.Saves, "TEST");
                Directory.CreateDirectory(Path.Combine(saves, "Cards"));
                File.WriteAllBytes(Path.Combine(saves, "plain.srm"), Bytes(16, i => i + 1));
                File.WriteAllBytes(Path.Combine(saves, "Cards", "mine.card"), Bytes(16, _ => 0xAB));
                var stored = new ConsolesConfig();
                stored.SetValue("TEST", "Wide", "true");
                stored.SetValue("TEST", "Ram", "9");
                stored.SetValue("TEST", "Cards", "named");
                stored.SetValue("TEST", "Card1", "mine");
                stored.SetValue("TEST", "ConsoleMemory", "shared");
                stored.Save();
                string own = Path.Combine(ConfigStore.Directory, "games", "test", "plain.json");
                Directory.CreateDirectory(Path.GetDirectoryName(own)!);
                File.WriteAllText(own, "{\"Rate\":\"48000\",\"Ram\":\"300\"}");
                Directory.CreateDirectory(DataStore.Firmware);
                File.WriteAllBytes(Path.Combine(DataStore.Firmware, "boot.rom"), Bytes(64, i => i));
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                string Info(uint s) => i.Read("session_info", rt, s);
                string o1 = i.Open(rt, plain);
                uint s1 = Id(o1);
                a.AddRange(new[] { o1, Info(s1), i.Read("save_media", rt, s1), i.Read("sessions", rt) });
                a.AddRange(new[] { i.DiscLid(rt, s1, true), Info(s1), i.DiscInsert(rt, s1, other), Info(s1), i.DiscChange(rt, s1, 0), Info(s1), i.DiscChange(rt, s1, 7) });
                a.AddRange(new[] { i.Call("disc_eject", rt, s1), Info(s1), i.DiscInsert(rt, s1, Path.Combine(dir, "absent.tst")), i.DiscInsert(rt, s1, null), i.DiscInsert(rt, s1, ""), Info(s1) });
                a.AddRange(new[] { i.Call("reset", rt, s1), Info(s1), i.DiscLid(rt, s1, false), Info(s1) });
                foreach ((string game, uint flags) in new[] { (card, 0u), (memory, CrystalAbi.OpenNoBattery), (memory, CrystalAbi.OpenSingleRows), (boot, 0u), (zip, 0u) })
                {
                    string o = i.Open(rt, game, flags: flags);
                    a.AddRange(new[] { o, Info(Id(o)), i.Read("save_media", rt, Id(o)) });
                }
                string o4 = i.Open(rt, cart);
                uint s4 = Id(o4);
                a.AddRange(new[] { o4, i.Call("disc_eject", rt, s4), i.DiscLid(rt, s4, true), i.DiscChange(rt, s4, 0), i.DiscInsert(rt, s4, other), Info(s4) });
                a.AddRange(new[] { i.Read("sessions", rt), Info(0), i.Call("close", rt, s4), i.Read("sessions", rt), Info(0), i.Call("close", rt, s4), i.Call("reset", rt, 99), Info(s4) });
                a.AddRange(new[]
                {
                    i.Open(rt, null), i.Open(rt, plain, console: "TEST"), i.Open(rt, Path.Combine(dir, "absent.tst")), i.Open(rt, plain, engine: "No Such Engine"),
                    i.Open(rt, plain, size: 40), i.Open(rt, plain, flags: 0x80), i.Open(rt, null, console: "Nowhere"), i.Open(rt, null, console: "test"),
                });
                foreach (uint s in new uint[] { 1, 2, 3, 4, 5, 6, 0 }) a.Add(i.Call("close", rt, s));
                a.Add(i.Read("sessions", rt));
                return a;
            });
        }

        [Fact]
        public void The_build_s_engines_open_games_and_empty_consoles_and_close_them_the_same()
        {
            string dir = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(dir);
            string W(string name, byte[] bytes)
            {
                string path = Path.Combine(dir, name);
                File.WriteAllBytes(path, bytes);
                return path;
            }
            string nes = W("Battery.nes", SyntheticNesRom.Build(prgBanks: 2, chrBanks: 0, mapper: 1, battery: true));
            string zipped = Path.Combine(dir, "Zipped.zip");
            using (var archive = ZipFile.Open(zipped, ZipArchiveMode.Create))
            using (var entry = archive.CreateEntry("inside.nes").Open()) entry.Write(SyntheticNesRom.Build());
            var games = new List<string>
            {
                nes, zipped, W("Game.sfc", SyntheticRom.Build()), W("Mono.gb", SyntheticGbRom.Build()), W("Colour.gbc", SyntheticGbRom.Build(cgbFlag: 0xC0)),
                W("Saves.z64", SyntheticN64Rom.BuildHomebrew(N64SaveType.Eeprom4k)), W("Game.md", SyntheticMdRom.Cartridge(saveBytes: 0x2000)),
                W("SegaCd.iso", SyntheticMdRom.Disc("U")), W("Psx.iso", SyntheticPsxDisc.Build().Iso),
            };
            string first = SyntheticPsxDisc.Build().WriteSet(dir, "Two (Disc 1)"), second = SyntheticPsxDisc.Build("Europe").WriteSet(dir, "Two (Disc 2)");
            string list = Path.Combine(dir, "Two.m3u");
            File.WriteAllText(list, "Two (Disc 1).cue\nTwo (Disc 2).cue\n");
            string lone = W("Lone.iso", SyntheticPsxDisc.Build("Japan").Iso);
            void Plant()
            {
                foreach (string ext in new[] { ".srm", ".sav" }) File.WriteAllBytes(Path.ChangeExtension(nes, ext), Bytes(0x2000, i => i * 7));
                var stored = new ConsolesConfig();
                stored.SetValue("PlayStation", "Cards", "named");
                stored.SetValue("PlayStation", "Card1", "mine");
                stored.SetValue("PlayStation", "Card2", "mine");
                stored.SetValue("Sega CD", "ConsoleMemory", "named");
                Directory.CreateDirectory(DataStore.Saves);
                File.WriteAllBytes(Path.Combine(DataStore.Saves, "Saves.srm"), Bytes(512, i => i ^ 0x5A));
                stored.Save();
            }
            SameWithFiles(BuildTree, Plant, (i, rt) =>
            {
                var a = new List<string>();
                foreach (string game in games)
                {
                    string o = i.Open(rt, game);
                    a.AddRange(new[] { o, i.Read("session_info", rt, Id(o)), i.Read("save_media", rt, Id(o)), i.Call("reset", rt, Id(o)), i.Call("close", rt, Id(o)) });
                }
                string m = i.Open(rt, list);
                uint s = Id(m);
                a.AddRange(new[] { m, i.Read("session_info", rt, s), i.DiscChange(rt, s, 1), i.Read("session_info", rt, s), i.DiscChange(rt, s, 2), i.DiscInsert(rt, s, lone) });
                a.AddRange(new[] { i.Read("session_info", rt, s), i.Call("disc_eject", rt, s), i.DiscLid(rt, s, false), i.Read("session_info", rt, s), i.Call("close", rt, s) });
                foreach (string console in new[] { "PlayStation", "psx", "Sega CD", "Nowhere", "SNES" })
                {
                    string o = i.Open(rt, null, console: console);
                    a.AddRange(new[] { o, i.Read("session_info", rt, Id(o)), i.Read("save_media", rt, Id(o)), i.Read("sessions", rt), i.Call("close", rt, Id(o)) });
                }
                string held = i.Open(rt, null, console: "PlayStation"), second = i.Open(rt, games[^1]);
                a.AddRange(new[] { held, second, i.Read("session_info", rt, Id(second)), i.Read("save_media", rt, Id(second)), i.Read("sessions", rt), i.Call("close", rt, Id(held)) });
                a.AddRange(new[] { i.Read("sessions", rt), i.Read("session_info", rt, 0), i.Call("close", rt, 0), i.Read("sessions", rt) });
                return a;
            });
        }
    }
}
