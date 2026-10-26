using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Nodes;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Scraping;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // Step 7's gate in both runtimes: a self-contained package carried to another machine, as its folder copied with its times and as its zip, opens there with its saves, states and their records, settings, art and patches - see EmuSen_GamePackages.md §19.
    public partial class CrystalParityTests
    {
        // A self-contained package for a library file, as pkg make --self-contained writes one: the game copied in with its time, the library file kept as its source.
        internal static void SelfContainedFor(string folder, string library, string rel, string system)
        {
            string game = Path.Combine(library, rel), copy = Path.Combine(folder, Path.GetFileName(game));
            Directory.CreateDirectory(folder);
            File.Copy(game, copy);
            File.SetLastWriteTimeUtc(copy, File.GetLastWriteTimeUtc(game));
            RomHashes h = RomHash.Of(game);
            JsonObject Seen(string at) => new() { ["size"] = new FileInfo(at).Length, ["mtime"] = GamePackagesTime(File.GetLastWriteTimeUtc(at)) };
            var p = GamePackage.Create(Path.GetFileNameWithoutExtension(game), system, "self-contained", new PackageFile("image", "package", Path.GetFileName(game), h.Size));
            ((JsonObject)p.Document["game"]!)["seen"] = Seen(copy);
            p.Document["dump"] = new JsonObject { ["file"] = new JsonObject { ["crc32"] = h.Crc32, ["md5"] = h.Md5, ["sha1"] = h.Sha1 } };
            p.Document["sources"] = new JsonArray(new JsonObject { ["role"] = "image", ["where"] = "library", ["path"] = rel, ["size"] = h.Size, ["seen"] = Seen(game), ["sha1"] = h.Sha1 });
            Assert.True(p.Write(Path.Combine(folder, GamePackage.FileName)));
        }

        // A folder copied as cp -rp copies it, or as a copy that loses every time.
        private static void CopyFolder(string from, string to, bool times = true)
        {
            foreach (string f in Directory.EnumerateFiles(from, "*", SearchOption.AllDirectories))
            {
                string target = Path.Combine(to, Path.GetRelativePath(from, f));
                Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                File.Copy(f, target);
                // File.Copy keeps the time, as Windows does, so a copy that loses it is made here.
                File.SetLastWriteTimeUtc(target, times ? File.GetLastWriteTimeUtc(f) : DateTime.UtcNow.AddMinutes(1));
            }
        }

        private static List<JsonElement> ToolRun(string request)
        {
            var events = new List<JsonElement>();
            using IToolJob job = LibraryToolJob.Start(request);
            var clock = System.Diagnostics.Stopwatch.StartNew();
            while (!job.Ended)
            {
                if (job.Next(200) is { } line) events.Add(JsonDocument.Parse(line).RootElement.Clone());
                Assert.True(clock.Elapsed < TimeSpan.FromMinutes(2), "the job did not end");
            }
            return events;
        }

        // A fresh machine: its own data root and settings, and a library of its own without the game.
        private string Machine(string name)
        {
            string root = Path.Combine(_root, name);
            DataStore.OverrideDirectory = Path.Combine(root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(root, "Config");
            string library = Path.Combine(root, "Library");
            Directory.CreateDirectory(library);
            new AppSettings { RomDirectory = library }.Save();
            return library;
        }

        [Fact]
        public void A_self_contained_package_carried_to_another_machine_as_its_folder_or_its_zip_opens_there_with_its_saves_states_settings_art_and_patches_both_ways()
        {
            Assert.True(LibraryToolJob.Available, LibraryToolJob.Report);
            string tree = TestTree(plain: false);
            string libraryA = Path.Combine(_root, "Library");
            string game = Path.Combine(libraryA, "TEST", "hack.tst");
            string packageA = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            byte[] original = CoreAdapterTests.Image(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
            byte[] patched = original.ToArray();
            patched[6 + 2] = 0x42;
            byte[] ips = Ips((8, new byte[] { 0x42 }));
            string key = ListKey(ips);
            Directory.CreateDirectory(Path.GetDirectoryName(game)!);
            File.WriteAllBytes(game, original);
            new AppSettings { RomDirectory = libraryA }.Save();
            SelfContainedFor(packageA, libraryA, "TEST/hack.tst", "test");
            PatchPackage(packageA, "hack.ips", ips, original, patched);
            string libraryBefore = TreeOf(libraryA);

            // Played on the first machine: the battery save written, slot 2 saved with its record, a setting of the game's own, and a cover.
            string slot2 = $"hack.patched-{key}.slot2.state", save = $"hack.patched-{key}.srm";
            Answers(CrystalImpl.Native, new[] { tree }, (i, rt) =>
            {
                uint s = Id(i.Open(rt, game));
                Assert.True(s != 999, "the library file opens the package's copy");
                i.Button(rt, s, 0, 8, true);
                i.Frame(rt, s);
                i.Button(rt, s, 0, 8, false);
                for (int f = 0; f < 6; f++) i.Frame(rt, s);
                i.SlotCall("state_slot_save", rt, s, 2);
                i.Call("close", rt, s);
                return new[] { i.GameSettingSet(rt, packageA, "Ram", "12") };
            });
            Directory.CreateDirectory(Path.Combine(packageA, "media", "covers"));
            File.WriteAllBytes(GamePackage.MediaPathFor(packageA, "covers", ".png"), new byte[] { 0x89, 0x50, 0x4E, 0x47, 1, 2, 3 });
            byte[] saved = File.ReadAllBytes(Path.Combine(packageA, "saves", save));
            Assert.True(File.Exists(Path.Combine(packageA, "states", Path.ChangeExtension(slot2, ".json"))), "the record is beside the state");
            Assert.True(File.Exists(Path.Combine(packageA, "settings.json")));

            // Its zip, written by the tool; read again here by System.IO.Compression, a reader of its own, with each file and its time.
            List<JsonElement> zipped = ToolRun("{\"op\":\"pkg-zip\",\"paths\":[\"TEST/hack.tst\"]}");
            string zip = GamePackages.ZipBeside(packageA);
            Assert.True(File.Exists(zip), string.Join("\n", zipped.Select(e => e.GetRawText())));
            using (ZipArchive archive = ZipFile.OpenRead(zip))
            {
                var names = archive.Entries.Select(e => e.FullName).ToHashSet();
                foreach (string f in Directory.EnumerateFiles(packageA, "*", SearchOption.AllDirectories))
                {
                    string name = "hack.tst/" + Path.GetRelativePath(packageA, f).Replace('\\', '/');
                    Assert.Contains(name, names);
                    ZipArchiveEntry entry = archive.GetEntry(name)!;
                    using var read = new MemoryStream();
                    entry.Open().CopyTo(read);
                    Assert.Equal(File.ReadAllBytes(f), read.ToArray());
                    Assert.True(Math.Abs((entry.LastWriteTime.UtcDateTime - File.GetLastWriteTimeUtc(f)).TotalSeconds) <= 2, name);
                }
            }
            Assert.Equal("hack.tst", GamePackages.ZipManifest(zip)?.Top);
            Assert.Equal(libraryBefore, TreeOf(libraryA));

            // Each runtime on the other machine: the package opens, its patch is applied, its save, slot and record, setting and art are there.
            IEnumerable<string> Opened(CrystalImpl i, nint rt, string opened, string package, bool record = true)
            {
                var a = new List<string>();
                uint s = Id(i.Open(rt, opened));
                Assert.True(s != 999, $"{opened} opened");
                string slots = Timeless(i.Read("state_slots", rt, s));
                Assert.Contains(Path.Combine(package, "states", slot2), slots);
                string recorded = Timeless(i.Read("state_record", rt, Path.Combine(package, "states", slot2)));
                Assert.Equal(record, recorded.Contains($"\"rom_md5\":\"{RomHash.Of(patched).Md5}\""));
                a.AddRange(new[] { slots.Replace(_root, "<root>"), recorded.Replace(_root, "<root>"), i.SlotCall("state_slot_load", rt, s, 2), i.SettingsGet(rt, s, 0, null, "Ram"), i.Call("close", rt, s) });
                Assert.Equal(saved, File.ReadAllBytes(Path.Combine(package, "saves", save)));
                return a;
            }
            string? Art(string library, string package)
            {
                RomLibraryResult scanned = RomLibrary.ScanManaged(library, null, RomLibrary.Beside.Tree);
                RomEntry entry = Assert.Single(scanned.Entries);
                Assert.Equal(package, entry.Package);
                var sources = new MediaSources((_, _) => null, DataStore.Media, null, null, packages: new Dictionary<string, string> { [entry.FullPath] = entry.Package! });
                (MediaSource source, string? path) = sources.Locate("test", entry.FullPath, ScrapeRules.Cover.EsdeType);
                Assert.Equal(MediaSource.Package, source);
                return path;
            }

            string libraryB = Machine("MachineB");
            string packageB = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            SameWithFiles(new[] { tree }, () =>
            {
                if (Directory.Exists(DataStore.Packages)) Directory.Delete(DataStore.Packages, recursive: true);
                CopyFolder(packageA, packageB);
            }, (i, rt) => Opened(i, rt, packageB, packageB), also: () => PackageTreeOf(DataStore.Packages));
            Assert.Equal(GamePackage.MediaPathFor(packageB, "covers", ".png"), Art(libraryB, packageB));

            // A copy that lost the times: the record drops out, the state still loads.
            Machine("MachineD");
            string packageD = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            SameWithFiles(new[] { tree }, () =>
            {
                if (Directory.Exists(DataStore.Packages)) Directory.Delete(DataStore.Packages, recursive: true);
                CopyFolder(packageA, packageD, times: false);
            }, (i, rt) => Opened(i, rt, packageD, packageD, record: false));

            // The zip on a third machine: listed as one entry, opened by its path, unpacked beside it once, and from then on its folder.
            string libraryC = Machine("MachineC");
            string zipC = Path.Combine(DataStore.Packages, "TEST", "hack.tst.zip"), packageC = Path.Combine(DataStore.Packages, "TEST", "hack.tst");
            SameWithFiles(new[] { tree }, () =>
            {
                if (Directory.Exists(DataStore.Packages)) Directory.Delete(DataStore.Packages, recursive: true);
                Directory.CreateDirectory(Path.GetDirectoryName(zipC)!);
                File.Copy(zip, zipC);
                Assert.Equal(zipC, Assert.Single(RomLibrary.ScanManaged(libraryC, null, RomLibrary.Beside.Tree).Entries).FullPath);
            }, (i, rt) => Opened(i, rt, zipC, packageC), also: () => PackageTreeOf(packageC));
            Assert.Equal(GamePackage.MediaPathFor(packageC, "covers", ".png"), Art(libraryC, packageC));
            foreach (string rel in new[] { slot2, Path.ChangeExtension(slot2, ".json") })
                Assert.Equal(File.GetLastWriteTimeUtc(Path.Combine(packageA, "states", rel)).Ticks, File.GetLastWriteTimeUtc(Path.Combine(packageC, "states", rel)).Ticks);
            Assert.Equal(packageC, GamePackages.Resolve(zipC, PackagePlaces.Current()).Package);
        }
    }
}
