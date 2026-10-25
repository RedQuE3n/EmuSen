using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // A patched game keeps its saves apart, decided 2026-10-25: each patch list reaches its own battery save and states, the unpatched game the original's, in both runtimes - see EmuSen_GamePackages.md §16.
    public partial class CrystalParityTests
    {
        // The key a list of IPS patches names its saves by, worked out here from §15.2's rule rather than asked of either runtime.
        private static string ListKey(params byte[][] patches) =>
            Hex(SHA1.HashData(Encoding.UTF8.GetBytes(string.Concat(patches.Select(p => $"ips {Hex(SHA1.HashData(p))}\n"))))).Substring(0, 8);

        [Fact]
        public void A_patched_game_keeps_its_own_battery_save_and_states_apart_from_the_original_s_both_ways()
        {
            string tree = TestTree(plain: false);
            string library = Path.Combine(_root, "Library");
            string game = Path.Combine(library, "TEST", "hack.tst");
            string package = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            byte[] original = CoreAdapterTests.Image(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
            byte[] targetA = original.ToArray(), targetB = original.ToArray();
            targetA[6 + 2] = 0x42;
            targetB[6 + 3] = 0x77;
            byte[] ipsA = Ips((8, new byte[] { 0x42 })), ipsB = Ips((9, new byte[] { 0x77 }));
            string keyA = ListKey(ipsA), keyB = ListKey(ipsB);
            string originalSave = Path.Combine(package, "saves", "hack.srm"), plainSave = Path.Combine(DataStore.Saves, "TEST", "hack.srm");
            string saveA = Path.Combine(package, "saves", $"hack.patched-{keyA}.srm"), saveB = Path.Combine(package, "saves", $"hack.patched-{keyB}.srm");
            byte[] kept = Bytes(16, i => 0xA0 + i), plainKept = Bytes(16, i => 0x30 + i);
            DateTime keptAt = new(2030, 1, 2, 3, 4, 5, DateTimeKind.Utc);
            void Plant()
            {
                foreach (string d in new[] { library, DataStore.Packages })
                    if (Directory.Exists(d)) Directory.Delete(d, recursive: true);
                Directory.CreateDirectory(Path.GetDirectoryName(game)!);
                File.WriteAllBytes(game, original);
                new AppSettings { RomDirectory = library }.Save();
                PackageFor(package, library, "TEST/hack.tst", "test");
                Directory.CreateDirectory(Path.GetDirectoryName(originalSave)!);
                File.WriteAllBytes(originalSave, kept);
                File.SetLastWriteTimeUtc(originalSave, keptAt);
                Directory.CreateDirectory(Path.GetDirectoryName(plainSave)!);
                File.WriteAllBytes(plainSave, plainKept);
                PatchPackage(package, "hack.ips", ipsA, original, targetA);
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                string Played(bool press)
                {
                    uint s = Id(i.Open(rt, package));
                    Assert.True(s != 999, "the package opened");
                    if (press) a.Add(i.Button(rt, s, 0, 8, true));
                    i.Frame(rt, s);
                    if (press) a.Add(i.Button(rt, s, 0, 8, false));
                    for (int f = 0; f < 6; f++) i.Frame(rt, s);
                    string slots = Timeless(i.Read("state_slots", rt, s));
                    a.AddRange(new[] { slots.Replace(_root, "<root>"), i.SlotCall("state_slot_save", rt, s, 2), i.Call("close", rt, s) });
                    return SlotPaths(slots);
                }
                string slotsA = Played(press: true);
                Assert.Contains(Path.Combine(package, "states", $"hack.patched-{keyA}.slot2.state"), slotsA);
                Assert.Contains(Path.Combine(package, "states", $"hack.patched-{keyA}.resume.state"), slotsA);
                Assert.True(File.Exists(saveA), "the patched game wrote its own save");
                Assert.NotEqual(kept, File.ReadAllBytes(saveA));
                Assert.Equal(kept, File.ReadAllBytes(originalSave));
                Assert.Equal(keptAt, File.GetLastWriteTimeUtc(originalSave));
                Assert.Equal(plainKept, File.ReadAllBytes(plainSave));
                PatchPackage(package, "hack.ips", ipsB, original, targetB);
                string slotsB = Played(press: true);
                Assert.Contains($"hack.patched-{keyB}.slot2.state", slotsB);
                Assert.Empty(slotsA.Split('\n').Intersect(slotsB.Split('\n')));
                Assert.True(File.Exists(saveB), "another list wrote another save");
                Assert.Equal(kept, File.ReadAllBytes(originalSave));
                GamePackage p = GamePackage.Read(Path.Combine(package, GamePackage.FileName));
                p.Document["patches"] = new JsonArray();
                p.Document["patched"] = null;
                Assert.True(p.Write(Path.Combine(package, GamePackage.FileName)));
                string unpatched = Played(press: false);
                Assert.Contains(Path.Combine(package, "states", "hack.slot2.state"), unpatched);
                Assert.DoesNotContain(".patched-", unpatched);
                Assert.Equal(kept, File.ReadAllBytes(originalSave));
                Assert.Equal(original, File.ReadAllBytes(game));
                return a;
            }, also: () => TreeOf(DataStore.Packages) + "\n--\n" + TreeOf(library));
        }
    }
}
