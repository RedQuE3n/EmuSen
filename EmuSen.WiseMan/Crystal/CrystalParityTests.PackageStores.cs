using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // The stores step 5 left, in both runtimes: a packaged game's state records and own settings live in its package, read alike by its plain path and by its package, and kept when the package moves - see EmuSen_GamePackages.md §18.
    public partial class CrystalParityTests
    {
        // A tree's files with their hashes, a package's state records with the moment, the build and the state's time hidden.
        private static string PackageTreeOf(string dir) => !Directory.Exists(dir) ? "" : string.Join("\n", Directory.EnumerateFiles(dir, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal)
            .Select(f => $"{Path.GetRelativePath(dir, f)} " + (Path.GetFileName(Path.GetDirectoryName(f)) == "states" && f.EndsWith(".json", StringComparison.Ordinal)
                ? Regex.Replace(Timeless(File.ReadAllText(f).Replace("\"SavedAt\"", "\"saved_at\"").Replace("\"Build\"", "\"build\"")), "\"StateModified\":[0-9]+", "\"StateModified\":n")
                : Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(f.EndsWith(".png") ? PixelsOf(f) : File.ReadAllBytes(f))))));

        [Fact]
        public void A_packaged_game_s_state_records_and_own_settings_are_its_package_s_by_either_path_and_kept_when_it_moves_both_ways()
        {
            string tree = TestTree(plain: false);
            string library = Path.Combine(_root, "Library");
            string game = Path.Combine(library, "TEST", "plain.tst");
            string package = Path.Combine(DataStore.Packages, "TEST", "plain.tst"), moved = Path.Combine(DataStore.Packages, "TEST", "Elsewhere", "plain.tst");
            string plainSettings = GameSettings.PlainPath("test", game);
            void Plant()
            {
                foreach (string d in new[] { library, DataStore.Packages, DataStore.Library })
                    if (Directory.Exists(d)) Directory.Delete(d, recursive: true);
                Directory.CreateDirectory(Path.GetDirectoryName(game)!);
                File.WriteAllBytes(game, CoreAdapterTests.Image(1, 2, 3, 4));
                // One time for both runs, so that the manifests each leaves record the same seen time.
                File.SetLastWriteTimeUtc(game, new DateTime(2026, 10, 25, 12, 0, 0, DateTimeKind.Utc));
                new AppSettings { RomDirectory = library }.Save();
                PackageFor(package, library, "TEST/plain.tst", "test");
                Directory.CreateDirectory(Path.GetDirectoryName(plainSettings)!);
                File.WriteAllText(plainSettings, "{\n  \"Wide\": \"true\"\n}");
            }
            SameWithFiles(new[] { tree }, Plant, (i, rt) =>
            {
                var a = new List<string>();
                string Record(string path) => Timeless(i.Read("state_record", rt, path)).Replace(_root, "<root>");
                // Before the package has settings of its own, the plain file's are read; the first write copies them in and leaves the plain file.
                a.AddRange(new[] { i.GameSettingGet(rt, game, "Wide"), i.GameSettingGet(rt, package, "Wide"), i.GameSettingSet(rt, game, "Ram", "12") });
                Assert.Equal("{\n  \"Wide\": \"true\"\n}", File.ReadAllText(plainSettings));
                Assert.Contains("\"Wide\": \"true\"", File.ReadAllText(Path.Combine(package, "settings.json")));
                a.AddRange(new[] { i.GameSettingGet(rt, game, "Ram"), i.GameSettingGet(rt, package, "Ram"), i.GameSettingGet(rt, package, "Wide") });
                uint s1 = Id(i.Open(rt, game));
                a.AddRange(new[] { i.SettingsGet(rt, s1, 0, null, "Ram"), i.SettingsSet(rt, s1, 0, null, "Wide", "false") });
                for (int f = 0; f < 12; f++) i.Frame(rt, s1);
                a.Add(i.SlotCall("state_slot_save", rt, s1, 2));
                string slot2 = Path.Combine(package, "states", "plain.slot2.state");
                Assert.True(File.Exists(Path.Combine(package, "states", "plain.slot2.json")), "the record is kept beside the state in its package");
                a.AddRange(new[] { Record(slot2), i.Call("close", rt, s1) });
                uint s2 = Id(i.Open(rt, package));
                a.AddRange(new[] { i.SettingsGet(rt, s2, 0, null, "Wide"), i.SettingsGet(rt, s2, 0, null, "Ram"), Record(slot2), i.SlotCall("state_slot_load", rt, s2, 2), i.Call("close", rt, s2) });
                Assert.Equal("{\n  \"Wide\": \"true\"\n}", File.ReadAllText(plainSettings));
                // The package moved: its records and settings go with it, by either path.
                Directory.CreateDirectory(Path.GetDirectoryName(moved)!);
                Directory.Move(package, moved);
                string movedSlot2 = Path.Combine(moved, "states", "plain.slot2.state");
                a.AddRange(new[] { Record(movedSlot2), i.GameSettingGet(rt, game, "Ram"), i.GameSettingGet(rt, moved, "Wide") });
                Assert.DoesNotContain("status", Record(movedSlot2));
                uint s3 = Id(i.Open(rt, game));
                a.AddRange(new[] { i.SettingsGet(rt, s3, 0, null, "Ram"), i.SlotCall("state_slot_load", rt, s3, 2), i.Call("close", rt, s3) });
                // A record whose state was rewritten by something else is no record; forgetting removes the package's.
                File.WriteAllBytes(movedSlot2, new byte[] { 1, 2, 3 });
                a.AddRange(new[] { Record(movedSlot2), i.Forget(rt, movedSlot2) });
                Assert.False(File.Exists(Path.Combine(moved, "states", "plain.slot2.json")));
                Directory.Move(moved, package);
                return a;
            }, also: () => PackageTreeOf(DataStore.Packages) + "\n--\n" + TreeOf(library) + "\n--\n" + TreeOf(Path.GetDirectoryName(plainSettings)!) + "\n--\n"
                + TreeOf(DataStore.Library).Split('\n').Count(l => l.StartsWith("states.db")));
        }
    }
}
