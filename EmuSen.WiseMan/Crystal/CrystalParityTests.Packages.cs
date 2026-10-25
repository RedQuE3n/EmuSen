using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json.Nodes;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // Step 5's gate in both runtimes: a game opened by its plain path and by its package reaches one battery save and one set of states, a moved game is found again by its hashes, and another dump is refused - see EmuSen_GamePackages.md §14.
    public partial class CrystalParityTests
    {
        // A linked package for a library file, as pkg make writes one.
        internal static void PackageFor(string folder, string library, string rel, string system)
        {
            string game = Path.Combine(library, rel);
            RomHashes h = RomHash.Of(game);
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), system, "linked", new PackageFile("image", "library", rel, h.Size));
            var seen = new FileInfo(game);
            ((JsonObject)p.Document["game"]!)["seen"] = new JsonObject { ["size"] = seen.Length, ["mtime"] = GamePackagesTime(seen.LastWriteTimeUtc) };
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
        }

        // The paths of a session's state slots, which two routes to one game must share.
        private static string SlotPaths(string slots) => string.Join("\n", JsonNode.Parse(slots)!.AsArray().Select(e => e!["path"]!.GetValue<string>()));

        private static string GamePackagesTime(DateTime utc) => new DateTime(utc.Ticks - utc.Ticks % TimeSpan.TicksPerSecond, DateTimeKind.Utc).ToString("yyyy-MM-dd'T'HH:mm:ss'Z'", System.Globalization.CultureInfo.InvariantCulture);

        [Fact]
        public void A_game_opened_by_its_plain_path_and_by_its_package_reaches_one_save_both_ways()
        {
            string tree = TestTree(plain: false);
            string library = Path.Combine(_root, "Library");
            string game = Path.Combine(library, "TEST", "plain.tst"), moved = Path.Combine(library, "TEST", "Elsewhere", "plain (moved).tst");
            string package = Path.Combine(DataStore.Packages, "TEST", "plain.tst");
            string plainSave = Path.Combine(DataStore.Saves, "TEST", "plain.srm"), packageSave = Path.Combine(package, "saves", "plain.srm");
            byte[] saved = Bytes(16, i => i + 1);
            void Plant()
            {
                foreach (string d in new[] { library, DataStore.Packages })
                    if (Directory.Exists(d)) Directory.Delete(d, recursive: true);
                Directory.CreateDirectory(Path.GetDirectoryName(game)!);
                File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
                new AppSettings { RomDirectory = library }.Save();
                PackageFor(package, library, "TEST/plain.tst", "test");
                Directory.CreateDirectory(Path.GetDirectoryName(plainSave)!);
                File.WriteAllBytes(plainSave, saved);
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                string Slots(uint s) => Timeless(i.Read("state_slots", rt, s));
                uint s1 = Id(i.Open(rt, game));
                string plainSlots = Slots(s1);
                a.AddRange(new[] { i.Read("session_info", rt, s1), plainSlots });
                Assert.Contains(Path.Combine(package, "states", "plain.state").Replace("\\", "\\\\"), plainSlots);
                for (int f = 0; f < 12; f++) i.Frame(rt, s1);
                a.AddRange(new[] { i.SlotCall("state_slot_save", rt, s1, 2), i.Call("close", rt, s1) });
                Assert.Equal(saved, File.ReadAllBytes(packageSave));
                Assert.Equal(saved, File.ReadAllBytes(plainSave));
                string o2 = i.Open(rt, package);
                uint s2 = Id(o2);
                Assert.True(s2 != 999, o2);
                Assert.Equal(SlotPaths(plainSlots), SlotPaths(Slots(s2)));
                a.AddRange(new[] { i.Read("session_info", rt, s2), Slots(s2), i.SlotCall("state_slot_load", rt, s2, 2), i.Call("close", rt, s2) });
                Directory.CreateDirectory(Path.GetDirectoryName(moved)!);
                File.Move(game, moved);
                string o3 = i.Open(rt, moved);
                uint s3 = Id(o3);
                Assert.True(s3 != 999, o3);
                string movedSlots = Slots(s3);
                a.AddRange(new[] { i.Read("session_info", rt, s3), movedSlots, i.Call("close", rt, s3) });
                Assert.Equal(SlotPaths(plainSlots), SlotPaths(movedSlots));
                Assert.Contains("TEST/Elsewhere/plain (moved).tst", File.ReadAllText(Path.Combine(package, GamePackage.FileName)));
                // Another dump now: by its plain path it plays as a plain game with its plain saves, and its package refuses (§14.8 Q34).
                File.WriteAllBytes(moved, CoreAdapterTests.Image(9, 9, 9, 9));
                File.SetLastWriteTimeUtc(moved, new DateTime(2031, 1, 1, 0, 0, 0, DateTimeKind.Utc));
                byte[] packageSaved = File.ReadAllBytes(packageSave);
                string o4 = i.Open(rt, moved);
                uint s4 = Id(o4);
                Assert.True(s4 != 999, o4);
                string changedSlots = Slots(s4);
                Assert.DoesNotContain(package.Replace("\\", "\\\\"), changedSlots);
                Assert.Contains(Path.Combine(DataStore.SaveStates, "plain (moved).slot2.state").Replace("\\", "\\\\"), changedSlots);
                a.AddRange(new[] { i.Read("session_info", rt, s4), changedSlots.Replace(_root, "<root>"), i.Button(rt, s4, 0, 8, true) });
                i.Frame(rt, s4);
                a.AddRange(new[] { i.Button(rt, s4, 0, 8, false), i.Call("close", rt, s4) });
                Assert.True(File.Exists(Path.Combine(DataStore.Saves, "TEST", "plain (moved).srm")), "the changed file wrote its plain save");
                Assert.Equal(packageSaved, File.ReadAllBytes(packageSave));
                string refused = i.Open(rt, package);
                Assert.Contains("has SHA-1", refused);
                a.Add(refused.Replace(_root, "<root>"));
                return a;
            }, also: () => PackageTreeOf(DataStore.Packages) + "\n--\n" + TreeOf(library));
        }
    }
}
